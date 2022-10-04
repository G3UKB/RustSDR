/*
app.rs

Module - app
manages startup, shutdown and object cache

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
pub mod dsp;

use std::thread;
use std::time::Duration;
use std::sync::Arc;
use std::option;

use socket2;
use crossbeam_channel::unbounded;

pub struct Appdata{
    pub i_sock : udp::udp_socket::Sockdata,
    pub p_sock : Arc<socket2::Socket>,
    //pub p_addr: option::Option<Arc<socket2::SockAddr>>,
    pub opt_udp_writer :  option::Option<udp::udp_writer::UDPWData>,
    pub opt_reader_join_handle: option::Option<thread::JoinHandle<()>>,
    pub i_hw_control : udp::hw_control::HWData,
    pub r_sender : crossbeam_channel::Sender<common::messages::ReaderMsg>,
    pub r_receiver : crossbeam_channel::Receiver<common::messages::ReaderMsg>,
    pub hw_sender : crossbeam_channel::Sender<common::messages::HWMsg>,
    pub hw_receiver : crossbeam_channel::Receiver<common::messages::HWMsg>,
    pub dsp_sender : crossbeam_channel::Sender<common::messages::DSPMsg>,
    pub dsp_receiver : crossbeam_channel::Receiver<common::messages::DSPMsg>,
}

impl Appdata {
    pub fn new() -> Appdata {
        // Create the message q's for reader, hardware and DSP
        let (r_s, r_r) = unbounded();
        let (hw_s, hw_r) = unbounded();
        let (dsp_s, dsp_r) = unbounded();

        // Create the shared socket
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

        // Create the UDP reader and writer
        let mut opt_udp_writer: option::Option<udp::udp_writer::UDPWData> = None;
        let mut opt_reader_join_handle: option::Option<thread::JoinHandle<()>> = None;
        let arc2 = p_sock.clone();
        let arc3 = p_sock.clone();
        match p_addr {
            Some(addr) => { 
                // Create UDP writer 
                let arc4 = addr.clone();
                let i_udp_writer = udp::udp_writer::UDPWData::new(arc2, arc4);
                opt_udp_writer = Some(i_udp_writer); 
                
                // Start the UDP reader thread
                opt_reader_join_handle = Some(udp::udp_reader::reader_start(r_r.clone(), arc3));
            },
            None => {
                println!("Address invalid, UDP reader and writer will not be started! Is hardware on-line?");
            }
        }

        Appdata { 
            i_sock : i_sock,
            p_sock : p_sock,
            //p_addr : p_addr,
            opt_udp_writer : opt_udp_writer,
            opt_reader_join_handle : opt_reader_join_handle,
            i_hw_control : i_hw_control,
            r_sender : r_s,
            r_receiver : r_r,
            hw_sender : hw_s,
            hw_receiver : hw_r,
            dsp_sender : dsp_s,
            dsp_receiver : dsp_r,
        }
    }

    pub fn app_init(&mut self) {
        println!("Initialising hardware...");
        //self.i_hw_control.do_start(false);
        //thread::sleep(Duration::from_millis(1000));
        // Call prime to init the hardware
        match &mut self.opt_udp_writer {
            Some(writer) => writer.prime(),  
            None => println!("Address invalid, hardware will not be primed!"),
        }
        thread::sleep(Duration::from_millis(1000));
        // Let the reader start
        self.r_sender.send(common::messages::ReaderMsg::StartListening).unwrap();
        //thread::sleep(Duration::from_millis(1000));

        self.i_hw_control.do_start(false);
        thread::sleep(Duration::from_millis(1000));
    }

    // Terminate the reader thread
    pub fn app_close(&mut self) { 
        
        if let Some(h) = self.opt_reader_join_handle.take(){
            self.r_sender.send(common::messages::ReaderMsg::StopListening).unwrap();
            thread::sleep(Duration::from_millis(1000));
            self.r_sender.send(common::messages::ReaderMsg::Terminate).unwrap();
            println!("Waiting for reader to terminate...");
            h.join();
            println!("Reader terminated");
        }

        // Stop the hardware
        self.i_hw_control.do_stop();
        thread::sleep(Duration::from_millis(10000));
        println!("Hardware stopped");
       
        thread::sleep(Duration::from_millis(10000));
    }
}
