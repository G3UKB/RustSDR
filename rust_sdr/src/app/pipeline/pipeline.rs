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
use std::io::{Read, Write};
use std::cell::RefCell;

use crate::app::common::messages;
use crate::app::common::common_defs;
use crate::app::common::ringb;
use crate::app::common::converters;
use crate::app::dsp;
use crate::app::udp::udp_writer;

enum ACTIONS {
    ActionNone,
    ActionTerm,
    ActionData,
}

//==================================================================================
// Runtime object for thread
pub struct PipelineData<'a>{
    receiver : crossbeam_channel::Receiver<messages::PipelineMsg>,
    rb_iq : &'a ringb::SyncByteRingBuf,
    iq_cond : &'a (Mutex<bool>, Condvar),
    rb_audio : &'a ringb::SyncByteRingBuf,
    rb_local_audio : &'a ringb::SyncByteRingBuf,
    iq_data : Vec<u8>,
    dec_iq_data : [f64; (common_defs::DSP_BLK_SZ * 2) as usize],
    proc_iq_data : [f64; (common_defs::DSP_BLK_SZ * 2) as usize],
    output_frame : [u8; common_defs::DSP_BLK_SZ as usize * 8],
    audio_frame : [u8; common_defs::DSP_BLK_SZ as usize * 8],
    run : bool,
    num_rx : u32,
}

// Implementation methods on UDPRData
impl PipelineData<'_> {
	// Create a new instance and initialise the default arrays
    pub fn new<'a> (
        receiver : crossbeam_channel::Receiver<messages::PipelineMsg>, 
        rb_iq : &'a ringb::SyncByteRingBuf, iq_cond : &'a (Mutex<bool>, Condvar),
        rb_audio : &'a ringb::SyncByteRingBuf, rb_local_audio : &'a ringb::SyncByteRingBuf) -> PipelineData {

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
            proc_iq_data : [0.0; (common_defs::DSP_BLK_SZ * 2) as usize],
            // Output contiguous audio and TX IQ data
            output_frame : [0; (common_defs::DSP_BLK_SZ as usize * 8) as usize],
            // Local audio out
            audio_frame : [0; (common_defs::DSP_BLK_SZ as usize * 8) as usize],
            run: false,
            // Until we have data set to 1
            num_rx: 1,
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
            /*
            if self.rb_iq.try_read().unwrap().available() >= (common_defs::DSP_BLK_SZ * common_defs::BYTES_PER_SAMPLE) as usize {
                // Enough data available
                let r = self.rb_iq.try_read();   
                match r {
                    Ok(mut m) => {
                        let iq_data = m.read(&mut self.iq_data);
                        match iq_data {
                            Ok(_sz) => {
                                action = ACTIONS::ActionData;
                                //println!("Read {:?} bytes from rb_iq", _sz);
                            }
                            Err(e) => println!("Read error on rb_iq {:?}. Skipping cycle.", e),
                        }
                    }
                    Err(e) => println!("Failed to get read lock on rb_iq [{:?}]. Skipping cycle.", e),
                }
            }
            */
            if self.rb_iq.read().available() >= (common_defs::DSP_BLK_SZ * common_defs::BYTES_PER_SAMPLE) as usize {
                let iq_data = self.rb_iq.read().read(&mut self.iq_data);
                match iq_data {
                    Ok(_sz) => {
                        action = ACTIONS::ActionData;
                        //println!("Read {:?} bytes from rb_iq", _sz);
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
        self.decode();
        let mut error: i32 = 0;
        dsp::dsp_interface::wdsp_exchange(0, &mut self.dec_iq_data,  &mut self.proc_iq_data, &mut error );
         if error == 0 {
            // We have output data from the DSP
            // Encode the data into a form suitable for the hardware
            self.encode_for_hardware();
            // Copy data to the output ring buffer 
            let r = self.rb_audio.write().write(&self.output_frame);
            match r {
                Err(e) => {
                    println!("Write error on rb_audio, skipping block {:?}", e);
                }
                Ok(_sz) => {
                    // We could signal data available but may not be necessary
                    // At the moment the writer thread just takes data when available
                }
            }
            // Now encode and copy data for local audio output
            self.encode_for_local_audio();
            // Copy data to the local audio ring buffer 
            let r = self.rb_local_audio.write().write(&self.audio_frame);
            match r {
                Err(e) => {
                    println!("Write error on rb_local_audio, skipping block {:?}", e);
                }
                Ok(_sz) => {
                    // We could signal data available but may not be necessary
                    // At the moment the writer thread just takes data when available
                }
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
        // Size to iterate over
        let sz: u32 = ((common_defs::DSP_BLK_SZ * common_defs::BYTES_PER_SAMPLE) - common_defs::BYTES_PER_SAMPLE);
        // Convert and scale input to output data.
        converters::i8be_to_f64le(&self.iq_data, &mut self.dec_iq_data, input_iq_scale, sz);

        /*
        // Iterate over each set of sample data
        // There are 3xI and 3xQ bytes for each receiver interleaved
        // Scale and convert each 24 bit value into the f64 array
        let mut raw: u32 = 0;
        let mut dec: u32 = 0;
        let mut as_int: i32;
        while raw <= ((common_defs::DSP_BLK_SZ * common_defs::BYTES_PER_SAMPLE) - common_defs::BYTES_PER_SAMPLE) {
            // Here we would iterate over each receiver and use a 2d array but for now one receiver
            // Pack the 3 x 8 bit BE into an int in LE
            as_int = ((
                    ((self.iq_data[(raw+2) as usize] as i32) << 8) | 
                    ((self.iq_data[(raw+1) as usize] as i32) << 16) | 
                    ((self.iq_data[raw as usize] as i32) << 24))
                    >>8);
            // Scale and write to target array
            self.dec_iq_data[dec as usize] = (as_int as f64) * input_iq_scale;

            raw += common_defs::BYTES_PER_SAMPLE;
            dec += 1;
        }
        */
    }

    // Encode the frame into a form suitable for the hardware
    fn encode_for_hardware(&mut self) {
        /*
        * The output data is structured as follows:
        * <L1><L0><R1><R0><I1><I0><Q1><Q0><L1><L0><R1><R0><I1><I0><Q1><Q0>...
        *
        * The L and R samples are sourced according to the audio output spec.
        */

        // Copy and encode the samples
		// proc_iq_data contains interleaved L/R double samples
		// proc_out_iq_data will contains interleaved I/Q double samples but no TX for now
		// output_frame is the output buffer to receive byte data in 16 bit big endian format
		// Both audio and IQ data are 16 bit values making 8 bytes in all

        // We get 1024 f64 samples interleaved left/right
        // We will get f64 float samples interleaves IQ output data
        // This means we have 1024*8*2 bytes of data to iterate on the input
        // However the output is 16 bit packed so we have 1024*2*2 to iterate on the output
        // Both in and out are interleaved which is the final factor of 2
        let out_sz: u32 = (common_defs::DSP_BLK_SZ * 4 * 2);
        let base: i32 = 2;
        let output_scale: f64 = base.pow(15) as f64;

        // Convert and scale input to output data.
        converters::f64le_to_i8be(&self.proc_iq_data, &mut self.output_frame, output_scale, out_sz);

        /* 
        let mut dest: usize = 0;
        let mut src: usize = 0;
        let mut L: i16;
        let mut R: i16;
        let mut I: i16;
        let mut Q: i16;
        
        // We iterate on the output side starting at the LSB
        while dest <= out_sz - 8 {
            L = (self.proc_iq_data[src] * output_scale) as i16;
            R = (self.proc_iq_data[src+1] * output_scale) as i16;
            I = 0 as i16;
            Q = 0 as i16;
            self.output_frame[dest] = ((L >> 8) & 0xff) as u8;
            self.output_frame[dest+1] = (L & 0xff) as u8;
            self.output_frame[dest+2] = ((R >> 8) & 0xff) as u8;
            self.output_frame[dest+3] = (R & 0xff) as u8;

            self.output_frame[dest+4] = I as u8;
            self.output_frame[dest+5] = I as u8;
            self.output_frame[dest+6] = Q as u8;
            self.output_frame[dest+7] = Q as u8;

            dest += 8;
            src += 2;
        }
        */
    }

    // Encode the frame into a form suitable for the hardware
    fn encode_for_local_audio(&mut self) {
        /*
        * The output data is structured as follows:
        * <L0><L1><L2><L3><R0><R1><R2><R3>...
        *
        * The L and R samples are in f32 format LE.
        */

        // Copy and encode the samples
		
        let out_sz: usize = (common_defs::DSP_BLK_SZ * 4 * 2) as usize;
        let mut dest: usize = 0;
        let mut src: usize = 0;
        let mut L: i32;
        let mut R: i32;
        let base: i32 = 2;
        let output_scale: f64 = base.pow(15) as f64;

        // We iterate on the output side starting at the MSB
        while dest <= out_sz - 8 {
            L = (self.proc_iq_data[src] * output_scale) as i32;
            R = (self.proc_iq_data[src+1] * output_scale) as i32;
            self.audio_frame[dest+3] = ((L >> 24) & 0xff) as u8;
            self.audio_frame[dest+2] = ((L >> 16) & 0xff) as u8;
            self.audio_frame[dest+1] = ((L >> 8) & 0xff) as u8;
            self.audio_frame[dest] = (L & 0xff) as u8;
            self.audio_frame[dest+3] = ((R >> 24) & 0xff) as u8;
            self.audio_frame[dest+2] = ((R >> 16) & 0xff) as u8;
            self.audio_frame[dest+1] = ((R >> 8) & 0xff) as u8;
            self.audio_frame[dest] = (R & 0xff) as u8;

            dest += 8;
            src += 2;
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
        pipeline_run(receiver, &rb_iq, &iq_cond, &rb_audio, &rb_local_audio);
    });
    return join_handle;
}

fn pipeline_run(
        receiver : crossbeam_channel::Receiver<messages::PipelineMsg>, 
        rb_iq : &ringb::SyncByteRingBuf, 
        iq_cond : &(Mutex<bool>, Condvar), 
        rb_audio : &ringb::SyncByteRingBuf,
        rb_local_audio : &ringb::SyncByteRingBuf){
    println!("Pipeline running");

    // Instantiate the runtime object
    let mut i_pipeline = PipelineData::new(receiver, rb_iq, iq_cond, rb_audio, rb_local_audio);

    // Exits when the reader loop exits
    i_pipeline.pipeline_run();

    println!("Pipeline exiting");
    thread::sleep(Duration::from_millis(1000));
}