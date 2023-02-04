/*
pipeline.rs

Manages the pipeline processing

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

use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex, Condvar};
use std::io::{Read, Write};

use crate::app::common::messages;
use crate::app::common::globals;
use crate::app::common::common_defs;
use crate::app::common::ringb;
use crate::app::common::converters;
use crate::app::dsp;

enum ACTIONS {
    ActionNone,
    ActionTerm,
    ActionData,
}

//==================================================================================
// Runtime object for thread
pub struct PipelineData{
    receiver : crossbeam_channel::Receiver<messages::PipelineMsg>,
    rb_iq : Arc<ringb::SyncByteRingBuf>,
    iq_cond : Arc<(Mutex<bool>, Condvar)>,
    rb_audio : Arc< ringb::SyncByteRingBuf>,
    rb_local_audio : Arc<ringb::SyncByteRingBuf>,
    iq_data : Vec<u8>,
    dec_iq_data : [f64; (common_defs::DSP_BLK_SZ * 2) as usize],
    disp_iq_data : [f32; (common_defs::DSP_BLK_SZ * 2) as usize],
    proc_iq_data : [f64; (common_defs::DSP_BLK_SZ * 2) as usize],
    output_frame : [u8; common_defs::DSP_BLK_SZ as usize * 8],
    audio_frame : [u8; common_defs::DSP_BLK_SZ as usize * 4],
    run : bool,
    #[allow(dead_code)]
    num_rx : u32,
}

// Implementation methods on UDPRData
impl PipelineData {
	// Create a new instance and initialise the default arrays
    pub fn new (
        receiver : crossbeam_channel::Receiver<messages::PipelineMsg>, 
        rb_iq : Arc<ringb::SyncByteRingBuf>, iq_cond : Arc<(Mutex<bool>, Condvar)>,
        rb_audio :Arc<ringb::SyncByteRingBuf>, rb_local_audio :Arc<ringb::SyncByteRingBuf>) -> PipelineData {

		PipelineData {
            receiver: receiver,
            rb_iq: rb_iq,
            iq_cond: iq_cond,
            rb_audio: rb_audio,
            rb_local_audio: rb_local_audio,
            // Read size from rb gives us 1024 samples interleaved
            iq_data: vec![0; (common_defs::DSP_BLK_SZ * common_defs::BYTES_PER_SAMPLE) as usize],
            // Exchange size with DSP is 1024 I and 1024 Q samples interleaved as f64
            dec_iq_data : [0.0; (common_defs::DSP_BLK_SZ * 2)as usize],
            disp_iq_data : [0.0; (common_defs::DSP_BLK_SZ * 2)as usize],
            proc_iq_data : [0.0; (common_defs::DSP_BLK_SZ * 2) as usize],
            // Output contiguous audio and TX IQ data
            output_frame : [0; (common_defs::DSP_BLK_SZ as usize * 8) as usize],
            // Local audio out
            audio_frame : [0; (common_defs::DSP_BLK_SZ as usize * 4) as usize],
            run: false,
            // Until we have data set to 1
            num_rx: globals::get_num_rx(),
		}
	}

    // Run loop for pipeline
    pub fn pipeline_run(&mut self) {
        loop {
            let action = self.prepare();
            match action {
                ACTIONS::ActionNone => (),
                ACTIONS::ActionTerm => break,
                ACTIONS::ActionData => self.sequence(),
            }
        }
    }

    // Lock and extract data from ring buffers if available.
    // Return ACTION to execute.
    // All locks are released at end of this fn. DO NOT put processing in here.
    fn prepare(&mut self) -> ACTIONS {
        // Wait for signal to start processing
        // A signal is given when a message or data is available
        let mut action = ACTIONS::ActionNone;
        let mut locked = self.iq_cond.0.lock().unwrap();
        let result = self.iq_cond.1.wait_timeout(locked, Duration::from_millis(100)).unwrap();
        locked = result.0;
        // Why were we woken?
        if *locked == true {
            // We were signaled so data available
            *locked = false;
            
            if self.rb_iq.read().available() >= (common_defs::DSP_BLK_SZ * common_defs::BYTES_PER_SAMPLE) as usize {
                let read_result = self.rb_iq.read().read(&mut self.iq_data);
                match read_result {
                    Ok(_sz) => {
                        action = ACTIONS::ActionData;
                    }
                    Err(e) => println!("Read error on rb_iq {:?}. Skipping cycle.", e),
                }
            }
        } else {
            // Timeout so check for messages
            let r = self.receiver.try_recv();
            match r {
                Ok(msg) => {
                    match msg {
                        messages::PipelineMsg::Terminate => action = ACTIONS::ActionTerm,
                        messages::PipelineMsg::StartPipeline => self.run = true,
                        messages::PipelineMsg::StopPipeline => self.run = false,
                    };
                },
                // Do nothing if there are no message matches
                _ => (),
            };
        }
        return action;
    }

    // Run the pipeline sequence
    fn sequence(&mut self) {
        // We just exchange for now
        // Convert and scale input to output data.
        converters::i8be_to_f64le(&self.iq_data, &mut self.dec_iq_data);
        let error: i32;

        // At 48K : 1024 in 1024 out
        // At 96K : 1024 in 512 out
        // At 102K : 1024 in 256 out
        let mut proc_iq_sz = self.proc_iq_data.len();
        let mut output_sz = self.output_frame.len();
        let mut audio_sz = self.audio_frame.len();
        if globals::get_smpl_rate() == common_defs::SMPLS_96K {
            proc_iq_sz = proc_iq_sz/2;
            output_sz = output_sz/2;
            audio_sz = audio_sz/2;
        } else if globals::get_smpl_rate() == common_defs::SMPLS_192K {
            proc_iq_sz = proc_iq_sz/4;
            output_sz = output_sz/4;
            audio_sz = audio_sz/4;
        }
        
        error = dsp::dsp_interface::wdsp_exchange(0, &mut self.dec_iq_data,  &mut self.proc_iq_data);
        for i in 0..proc_iq_sz {
            self.proc_iq_data[i] = self.proc_iq_data[i] * 0.2;
            if self.proc_iq_data[i]  > 1.0 {
                self.proc_iq_data[i] = 1.0;
            }
            if self.proc_iq_data[i]  < -1.0 {
                self.proc_iq_data[i] = -1.0;
            }
        }

        // Pass data to spectrum
        for i in 0..self.dec_iq_data.len() {
            self.disp_iq_data[i] = self.dec_iq_data[i] as f32;
        }
        dsp::dsp_interface::wdsp_write_spec_data(0, &mut self.disp_iq_data);
        
        // Process IQ data
        if error == 0 {
            // We have output data from the DSP
            // Encode the data into a form suitable for the hardware
            // Convert and scale input to output data.
            converters::f64le_to_i8be(output_sz, &self.proc_iq_data, &mut self.output_frame);
            // Copy data to the output ring buffer
            let mut v_output_frame: Vec<u8> = self.output_frame.to_vec();
            v_output_frame.resize(output_sz, 0);
            let r = self.rb_audio.write().write(&v_output_frame);
            match r {
                Err(_e) => {
                    // UDP writer not ready yet. Try next time.
                    //println!("Write error on rb_audio, skipping block {:?}", e);
                }
                Ok(_sz) => {
                    // We could signal data available but may not be necessary
                    // At the moment the writer thread just takes data when available
                }
            }
            // Now encode and copy data for local audio output
            // Convert and scale input to output data.
            converters::f64le_to_i8le(audio_sz, &self.proc_iq_data, &mut self.audio_frame);
            // Copy data to the local audio ring buffer 
            let mut v_audio_frame = self.audio_frame.to_vec();
            v_audio_frame.resize(audio_sz, 0);
            let r = self.rb_local_audio.write().write(&v_audio_frame);
            match r {
                Err(_e) => {
                    // Audio system not up yet. Try next time.
                    //println!("Write error on rb_local_audio, skipping block {:?}", e);
                }
                Ok(_sz) => {
                    // We could signal data available but may not be necessary
                    // At the moment the writer thread just takes data when available
                }
            }
        } else {
            println!("DSP returned an error, starvation!");
        }
    }
}

//==================================================================================
// Thread startup
pub fn pipeline_start(
    receiver : crossbeam_channel::Receiver<messages::PipelineMsg>, 
    rb_iq : Arc<ringb::SyncByteRingBuf>,
    iq_cond : Arc<(Mutex<bool>, Condvar)>,
    rb_audio : Arc<ringb::SyncByteRingBuf>,
    rb_local_audio : Arc<ringb::SyncByteRingBuf>) -> thread::JoinHandle<()> {
    let join_handle = thread::spawn(  move || {
        pipeline_run(receiver, rb_iq, iq_cond, rb_audio, rb_local_audio);
    });
    return join_handle;
}

fn pipeline_run(
        receiver : crossbeam_channel::Receiver<messages::PipelineMsg>, 
        rb_iq : Arc<ringb::SyncByteRingBuf>, 
        iq_cond : Arc<(Mutex<bool>, Condvar)>, 
        rb_audio : Arc<ringb::SyncByteRingBuf>,
        rb_local_audio : Arc<ringb::SyncByteRingBuf>){
    println!("Pipeline running");

    // Instantiate the runtime object
    let mut i_pipeline = PipelineData::new(receiver,rb_iq, iq_cond, rb_audio, rb_local_audio);

    // Exits when the reader loop exits
    i_pipeline.pipeline_run();

    println!("Pipeline exiting");
    thread::sleep(Duration::from_millis(1000));
}