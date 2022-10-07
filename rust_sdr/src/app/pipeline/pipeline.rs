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
use std::sync::{Arc, Mutex, Condvar, MutexGuard, WaitTimeoutResult};
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
    dec_iq_data : [f32; common_defs::NUM_SMPLS_1_RADIO as usize],
    run : bool,
    num_rx : u32,
}

// Implementation methods on UDPRData
impl PipelineData<'_> {
	// Create a new instance and initialise the default arrays
    pub fn new<'a> (receiver : crossbeam_channel::Receiver<messages::PipelineMsg>, rb_iq : &'a ringb::SyncByteRingBuf, iq_cond : &'a (Mutex<bool>, Condvar)) -> PipelineData {

		PipelineData {
            receiver: receiver,
            rb_iq: rb_iq,
            iq_data: vec![0; (common_defs::PROT_SZ * 2) as usize],
            // Allow for one receiver until we enhance to multiple
            dec_iq_data : [0.0; (common_defs::NUM_SMPLS_1_RADIO) as usize],
            iq_cond: iq_cond,
            run: false,
            // Until we have data set to 1
            num_rx: 1,
		}
	}

    // Run loop for pipeline
    pub fn pipeline_run(&mut self) {
        loop {
            // Wait for signal to start processing
            // A signal is given when a message or data is available
            let mut locked = self.iq_cond.0.lock().unwrap();
            let result = self.iq_cond.1.wait_timeout(locked, Duration::from_millis(100)).unwrap();
            locked = result.0;
                // Execution of pipeline tasks
                // Read IQ data (TDB read Mic data)
                if *locked == true {
                    *locked = false;
                    let r = self.rb_iq.try_read();   
                    match r {
                        Ok(mut m) => {
                            let iq_data = m.read(&mut self.iq_data);
                            match iq_data {
                                Ok(sz) => (), //println!("Read {:?} bytes from rb_iq", sz),
                                Err(e) => println!("Error on read {:?} from rb_iq", e),
                            }
                        }
                        Err(e) => println!("Failed to get read lock on rb_iq [{:?}]. Skipping cycle.", e),
                    }
                } else {
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
                }
            }
        }

        // Decode the frame into a form suitable for signal processing
        fn decode(&mut self) {
        /*
        *
        * Each IQ block is formatted as follows:
        *	For 1 receiver:
        *	0                        ... in_iq_sz
        *	<I2><I1><I0><Q2><Q1><Q0>
        *	For 2 receivers:
        *	0                        					... in_iq_sz
        *    <I12><I11><I10><Q12><Q11><Q10><I22><I21><I20><Q22><Q21><Q20>
        *	etc to 8 receivers
        *	The output is interleaved IQ per receiver.
        *
        * Each Mic block is formatted as follows:
        *	0                        ... in_mic_sz
        *	<M1><M0><M1><M0>
        */

        // We move data from the iq_data vec to the dec_iq_data array ready to FFI to DSP.
        // We also scale the data and convert from big endian to little endian as the hardware
        // uses big endian format.

        // Scale factors
        let base: i32 = 2;
        let input_iq_scale: f64 = 1.0 /(base.pow(23)) as f64;

        // Iterate over each set of sample data
	    // There are 3xI and 3xQ bytes for each receiver interleaved
        // Scale and convert each 24 bit value into the f32 array
        let mut raw: u32 = 0;
        while raw <= (common_defs::PROT_SZ * 2_ - common_defs::BYTES_PER_SAMPLE) {
            // Here we would iterate over each receiver and use a 2d array but for now one receiver


            raw += common_defs::BYTES_PER_SAMPLE;
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