/*
app.rs

Module - app
Manages startup, shutdown and object cache

Copyright (C) 2022 by G3UKB Bob Cowdery

This program is free software; you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation; either version 2 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program; if not, write to the Free Software
Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA

The authors can be reached by email at:

bob@bobcowdery.plus.com
*/

pub mod common;
pub mod udp;
pub mod protocol;
pub mod pipeline;
pub mod dsp;
pub mod audio;
pub mod ui;
use crate::app::common::common_defs;

use std::sync::{Arc, Mutex, Condvar};
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::thread;
use std::time::Duration;
use std::option;
use eframe::egui;


use socket2;
use crossbeam_channel::unbounded;

//=========================================================================================
// Object store for the entire system level 1
// Objects down the tree instantiate local objects as required
pub struct Appdata{
    //=================================================
    // UDP module related
    // UDP socket
    pub i_sock : udp::udp_socket::Sockdata,
    pub p_sock : Arc<socket2::Socket>,

    // UDP Reader and Writer
    // Writer thread join handle
    pub opt_writer_join_handle : option::Option<thread::JoinHandle<()>>,
    // Channel
    pub w_sender : crossbeam_channel::Sender<common::messages::WriterMsg>,
    pub w_receiver : crossbeam_channel::Receiver<common::messages::WriterMsg>,
    // Reader thread join handle
    pub opt_reader_join_handle: option::Option<thread::JoinHandle<()>>,
    // Channel
    pub r_sender : crossbeam_channel::Sender<common::messages::ReaderMsg>,
    pub r_receiver : crossbeam_channel::Receiver<common::messages::ReaderMsg>,

    // Hardware controller
    pub i_hw_control : udp::hw_control::HWData,
    // Channel
    pub hw_sender : crossbeam_channel::Sender<common::messages::HWMsg>,
    pub hw_receiver : crossbeam_channel::Receiver<common::messages::HWMsg>,

    // Local audio
    pub i_local_audio : audio::audio_out::AudioData,
    pub stream : Option<cpal::Stream>,

    //=================================================
    // Pipeline
    // Channel
    pub pipeline_sender : crossbeam_channel::Sender<common::messages::PipelineMsg>,
    pub pipeline_receiver : crossbeam_channel::Receiver<common::messages::PipelineMsg>,
    // DSP thread join handle
    pub opt_pipeline_join_handle: option::Option<thread::JoinHandle<()>>,
    // Ring buffer Reader thread <-> pipeline thread
    pub rb_iq : Arc<common::ringb::SyncByteRingBuf>,

    // Command and Control out
    pub i_cc : Arc<Mutex<protocol::cc_out::CCDataMutex>>,

    //=================================================
    // State
    pub run : bool,

    //=================================================
    // User interface object
    //pub i_ui : ui::main_window::UIState,
}

//=========================================================================================
// Implementation
impl Appdata {
    // Instantiate the application modules
    pub fn new() -> Appdata {
        // Local runnable
        let mut l_run = false;

        // First check/create the DSP Wisdom file
        dsp::dsp_interface::wdsp_wisdom();

        // Open a DSP receiver channel
        dsp::dsp_interface::wdsp_open_ch(
            common::common_defs::CH_RX as i32, 0, common::common_defs::DSP_BLK_SZ as i32, 
            common::common_defs::DSP_BLK_SZ as i32, common::common_defs::SMPLS_48K as i32, 
            common::common_defs::SMPLS_48K as i32, 0.0, 0.0, 0.0, 0.0);
        // and start the channel
        dsp::dsp_interface::wdsp_set_ch_state(0, 1, 0);

        // TBD same for TX channel

        // Open a display channel
        // Need to be set dynamically
        const FFT_SZ: i32 = 8192;
        const SUB_SPANS: i32 = 1;
        const OVER_FRAMES: i32 = 10;
        const FRAME_RATE: i32 = 20;
        const SAMPLE_RATE: i32 = 48000;
        const IN_SZ: i32 = 1024;
        const DISPLAY_WIDTH: i32 = 300;
        if dsp::dsp_interface::wdsp_open_disp(
            0, FFT_SZ, common_defs::WINDOW_TYPES::RECTANGULAR as i32, 
            SUB_SPANS, IN_SZ, DISPLAY_WIDTH, 
            common_defs::AV_MODE::PAN_TIME_AV_LIN as i32, OVER_FRAMES, 
            SAMPLE_RATE, FRAME_RATE) {
                println!("Opened display channel");
        }

        // Create the message q's for reader, hardware and Pipeline
        let (r_s, r_r) = unbounded();
        let (w_s, w_r) = unbounded();
        let (hw_s, hw_r) = unbounded();
        let (pipeline_s, pipeline_r) = unbounded();

        // Create ring buffers 
        // Buffer for read IQ data to DSP
        let num_rx = 1; // Until this is set
        let rb_capacity: usize = (num_rx * common::common_defs::PROT_SZ * 2 * common::common_defs::BYTES_PER_SAMPLE * common::common_defs::FRAMES_IN_RING ) as usize;
        let rb_iq = Arc::new(common::ringb::SyncByteRingBuf::with_capacity(rb_capacity));
        // Buffer to write audio data from DSP
        let rb_audio = Arc::new(common::ringb::SyncByteRingBuf::with_capacity(rb_capacity));
        // Buffer to write audio data from DSP for local audio
        let rb_local_audio = Arc::new(common::ringb::SyncByteRingBuf::with_capacity(rb_capacity));

        // Create condition variables
        // Between UDP Reader and Pipeline for data transfer
        let iq_cond = Arc::new((Mutex::new(false), Condvar::new()));
        let audio_cond = Arc::new((Mutex::new(false), Condvar::new()));

        // Create the shared socket, initially as a broadcast socket for discovery
        let mut i_sock = udp::udp_socket::Sockdata::new();
        let p_sock = i_sock.udp_sock_ref();

        // Create hardware control
        let arc1 = p_sock.clone();
        let mut i_hw_control = udp::hw_control::HWData::new(arc1);
        // Do discovery and get address of the hardware unit
        if !i_hw_control.do_discover() {
            println!("Discovery failed, reader and writer will not be operational!");
        }
        let p_addr: option::Option<Arc<socket2::SockAddr>> = i_hw_control.udp_addr_ref();
        // Revert the socket to non-broadcast and set buffer size
        i_sock.udp_revert_socket();

        // Create an instance of the cc_out type
        let mut i_cc = Arc::new(Mutex::new(protocol::cc_out::CCDataMutex::new()));

        // Create the UDP reader and writer if we have a valid hardware address
        let mut opt_udp_writer: option::Option<udp::udp_writer::UDPWData> = None;
        let mut opt_reader_join_handle: option::Option<thread::JoinHandle<()>> = None;
        let mut opt_writer_join_handle: option::Option<thread::JoinHandle<()>> = None;
        let arc2 = p_sock.clone();
        let arc3 = p_sock.clone();
        let arc4 = p_sock.clone();
        match p_addr {
            Some(addr) => { 
                // Create UDP writer 
                let arc5 = addr.clone();
                
                // Start the UDP writer thread
                opt_writer_join_handle = Some(
                    udp::udp_writer::writer_start(w_r.clone(), 
                    arc3, arc5, 
                    rb_audio.clone(), audio_cond.clone(), i_cc.clone()));

                // Start the UDP reader thread
                opt_reader_join_handle = Some(udp::udp_reader::reader_start(r_r.clone(), arc4, rb_iq.clone(), iq_cond.clone()));

                // OK to run
                l_run = true;
            },
            None => {
                println!("Address invalid, UDP reader and writer will not be started! Is hardware on-line?");
            }
        }

        // Start the pipeline thread
        let mut opt_pipeline_join_handle: option::Option<thread::JoinHandle<()>> = None;
        opt_pipeline_join_handle = Some(pipeline::pipeline::pipeline_start(
                pipeline_r.clone(), rb_iq.clone(), iq_cond.clone(), rb_audio.clone(), rb_local_audio.clone()));

        // Create the local audio
        let mut i_local_audio = audio::audio_out::AudioData::new(rb_local_audio.clone());

        // Create the UI instance
        //let mut i_ui: ui::main_window::UIState = ui::main_window::UIState::new(i_cc.clone());

        // Initialise the application data
        Appdata { 
            i_sock : i_sock,
            p_sock : p_sock,
            opt_writer_join_handle : opt_writer_join_handle,
            opt_reader_join_handle : opt_reader_join_handle,
            r_sender : r_s,
            r_receiver : r_r,
            w_sender : w_s,
            w_receiver : w_r,
            i_hw_control : i_hw_control,
            hw_sender : hw_s,
            hw_receiver : hw_r,
            pipeline_sender : pipeline_s,
            pipeline_receiver : pipeline_r,
            opt_pipeline_join_handle : opt_pipeline_join_handle,
            rb_iq : rb_iq,
            i_local_audio : i_local_audio,
            stream : None,
            run : l_run,
            i_cc : i_cc,
            //i_ui : i_ui,
        }
    }
    
    //=========================================================================================
    // Initialise system to a running state
    pub fn app_init(&mut self) {

        // Prime the hardware.
        self.w_sender.send(common::messages::WriterMsg::PrimeHardware).unwrap();

        // Start the pipeline. Waits for data available signal.
        self.pipeline_sender.send(common::messages::PipelineMsg::StartPipeline).unwrap();

        // Start the UDP reader. Will wait for UDP data when hardware starts
        // then signals the pipeline
        self.r_sender.send(common::messages::ReaderMsg::StartListening).unwrap();

        if self.run {
            // Start the hardware IQ stream and optional wide band data.
            self.i_hw_control.do_start(false);

            // Start the local audio stream
            self.stream = Some(self.i_local_audio.run_audio());
        }
    }

    //=========================================================================================
    // Run the UI event loop. Only returns when the UI is closed.
    pub fn ui_run(&mut self) {
        //self.i_ui.run_event_loop();
        let options = eframe::NativeOptions::default();
        let i_cc = self.i_cc.clone();
        eframe::run_native(
            "Rust SDR",
            options,
            Box::new(|cc| Box::new(ui::egui_ui::UIApp::new(cc, i_cc))),
        );
    }

    //=========================================================================================
    // Tidy close everything
    pub fn app_close(&mut self) { 
        
        if self.run {
            // Close local audio
            self.i_local_audio.close_audio(&(self.stream.as_ref().unwrap()));

            // Tell threads to stop
            self.r_sender.send(common::messages::ReaderMsg::StopListening).unwrap();
            self.w_sender.send(common::messages::WriterMsg::Terminate).unwrap();
            self.r_sender.send(common::messages::ReaderMsg::Terminate).unwrap();

            // Wait for UDP writer to exit
            if let Some(h) = self.opt_writer_join_handle.take(){
                println!("Waiting for writer to terminate...");
                h.join().expect("Join UDP Writer failed!");
                println!("Writer terminated");
            }

            // Wait for UDP reader to exit
            if let Some(h) = self.opt_reader_join_handle.take(){
                println!("Waiting for reader to terminate...");
                h.join().expect("Join UDP Reader failed!");
                println!("Reader terminated");
            }
        }

        // Terminate pipeline
        self.pipeline_sender.send(common::messages::PipelineMsg::Terminate).unwrap();
        // Wait for pipeline to exit
        if let Some(h) = self.opt_pipeline_join_handle.take(){
            println!("Waiting for pipeline to terminate...");
            h.join().expect("Join Pipeline failed!");;
            println!("Pipeline terminated");
        }
        
        if self.run {
            // Stop the hardware
            self.i_hw_control.do_stop();
            println!("Hardware stopped");
        }
       
    }
}
