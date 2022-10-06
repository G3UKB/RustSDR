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
use std::io::{self, Read, Write};

use crate::app::common::messages;
use crate::app::common::common_defs;
use crate::app::common::ringb;

//==================================================================================
// Runtime object for thread
pub struct PipelineData<'a>{
    receiver : crossbeam_channel::Receiver<messages::PipelineMsg>,
    rb_iq : &'a ringb::SyncByteRingBuf,
    iq_cond : &'a (Mutex<bool>, Condvar),
    iq_data : Vec<u8>,
    run : bool,
}

// Implementation methods on UDPRData
impl PipelineData<'_> {
	// Create a new instance and initialise the default arrays
    pub fn new<'a> (receiver : crossbeam_channel::Receiver<messages::PipelineMsg>, rb_iq : &'a ringb::SyncByteRingBuf, iq_cond : &'a (Mutex<bool>, Condvar)) -> PipelineData {

		PipelineData {
            receiver: receiver,
            rb_iq: rb_iq,
            iq_data : vec![0; (common_defs::PROT_SZ * 2) as usize],
            iq_cond : iq_cond,
            run: false,
		}
	}

    // Run loop for DSP
    pub fn pipeline_run(&mut self) {
        loop {
            // Wait for signal to start processing
            // A signal is given when a message or data is available
            let mut running = self.iq_cond.0.lock().unwrap();
            self.iq_cond.1.wait(running);
            
            // Check for messages
            let r = self.receiver.try_recv();
            match r {
                Ok(msg) => {
                    match msg {
                        messages::PipelineMsg::Terminate => break,
                        messages::PipelineMsg::StartPipeline => self.run = true,
                        messages::PipelineMsg::StopPipeline => self.run = false,
                    };
                },
                // Do nothing if there are no message matches
                _ => (),
            };

            // Execution of pipeline tasks
            // Read IQ data (TDB read Mic data)
            let r = self.rb_iq.try_read();   
            match r {
                Ok(mut m) => {
                    let iq_data = m.read(&mut self.iq_data);
                    match iq_data {
                        Ok(sz) => println!("Read {:?} bytes from rb_iq", sz),
                        Err(e) => println!("Error on read {:?} from rb_iq", e),
                    }
                }
                Err(e) => println!("Failed to get read lock on rb_iq{:?}", e),
            }
        }
    }
}

//==================================================================================
// Thread startup
pub fn pipeline_start(
    receiver : crossbeam_channel::Receiver<messages::PipelineMsg>, 
    rb_iq : Arc<ringb::SyncByteRingBuf>,
    iq_cond : Arc<(Mutex<bool>, Condvar)>) -> thread::JoinHandle<()>{
    let join_handle = thread::spawn(  move || {
        pipeline_run(receiver, &rb_iq, &iq_cond);
    });
    return join_handle;
}

fn pipeline_run(receiver : crossbeam_channel::Receiver<messages::PipelineMsg>, rb_iq : &ringb::SyncByteRingBuf, iq_cond : &(Mutex<bool>, Condvar)) {
    println!("Pipeline running");

    // Instantiate the runtime object
    let mut i_pipeline = PipelineData::new(receiver, rb_iq, iq_cond);

    // Exits when the reader loop exits
    i_pipeline.pipeline_run();

    println!("Pipeline exiting");
    thread::sleep(Duration::from_millis(1000));
}