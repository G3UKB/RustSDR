/*
dsp_interface.rs

FFI interface to DSP lib

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
use std::sync::Arc;

use crate::app::common::messages;
use crate::app::common::ringb;

//==================================================================================
// Runtime object for thread
pub struct DSPData{
    receiver : crossbeam_channel::Receiver<messages::DSPMsg>,
    rb_iq : &ringb::SyncByteRingBuf,
}

// Implementation methods on UDPRData
impl DSPData {
	// Create a new instance and initialise the default arrays
    pub fn new(receiver : crossbeam_channel::Receiver<messages::DSPMsg>, rb_iq : &ringb::SyncByteRingBuf) -> DSPData {

		DSPData {
            receiver: receiver,
            rb_iq: rb_iq,
		}
	}

    // Run loop for DSP
    pub fn dsp_run(&mut self) {
        loop {
            // Check for messages
            let r = self.receiver.try_recv();
            match r {
                Ok(msg) => {
                    match msg {
                        messages::DSPMsg::Terminate => break,
                    };
                },
                // Do nothing if there are no message matches
                _ => (),
            };
            thread::sleep(Duration::from_millis(100));
        }
    }
}

//==================================================================================
// Thread startup
pub fn dsp_start(receiver : crossbeam_channel::Receiver<messages::DSPMsg>, rb_iq : Arc<ringb::SyncByteRingBuf>) -> thread::JoinHandle<()>{
    let join_handle = thread::spawn(  move || {
        reader_run(receiver, &rb_iq);
    });
    return join_handle;
}

fn reader_run(receiver : crossbeam_channel::Receiver<messages::DSPMsg>, rb_iq : &ringb::SyncByteRingBuf) {
    println!("DSP Interface running");

    // Instantiate the runtime object
    let mut i_interface = DSPData::new(receiver, rb_iq);

    // Exits when the reader loop exits
    i_interface.dsp_run();

    println!("DSP Interface exiting");
    thread::sleep(Duration::from_millis(1000));
}