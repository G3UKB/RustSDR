/*
udp_reader.rs

Module - udp_reader
Manages read data over UDP from the SDR hardware

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
use std::mem::MaybeUninit;
use std::sync::{Arc, Mutex, Condvar};
use std::io::Write;

use socket2;

use crate::app::protocol;
use crate::app::common::ringb;
use crate::app::common::globals;
use crate::app::common::common_defs;
use crate::app::common::messages;

//==================================================================================
// Runtime object for thread
pub struct UDPRData{
    receiver : crossbeam_channel::Receiver<messages::ReaderMsg>,
	p_sock :  Arc<socket2::Socket>,
    rb_iq : Arc<ringb::SyncByteRingBuf>,
    iq_cond : Arc<(Mutex<bool>, Condvar)>,
    udp_frame : [MaybeUninit<u8>; common_defs::FRAME_SZ as usize],
    pub i_seq: protocol::seq_in::SeqData,
    listen: bool,
    iq: [u8; common_defs::IQ_ARR_SZ_R1 as usize],
    mic: [u8; common_defs::MIC_ARR_SZ_R1 as usize],
}

// Implementation methods on UDPRData
impl UDPRData {
	// Create a new instance and initialise the default arrays
    pub fn new(
        receiver : crossbeam_channel::Receiver<messages::ReaderMsg>, 
        p_sock : Arc<socket2::Socket>, 
        rb_iq : Arc<ringb::SyncByteRingBuf>,
        iq_cond : Arc<(Mutex<bool>, Condvar)>) -> UDPRData {
        // Create an instance of the sequence type
        let i_seq = protocol::seq_in::SeqData::new();

		UDPRData {
            receiver: receiver,
			p_sock: p_sock,
            rb_iq : rb_iq,
            iq_cond : iq_cond,
            // Received UDP data buffer
            udp_frame: [MaybeUninit::uninit(); common_defs::FRAME_SZ as usize],
            // UDP data contains a header + 2 protocol frames
            i_seq: i_seq,
            listen: false,
            iq: [0; common_defs::IQ_ARR_SZ_R1 as usize],
            mic: [0; common_defs::MIC_ARR_SZ_R1 as usize],
		}
	}

    // This is the thread main loop. When this exits the thread exits.
    pub fn reader_run(&mut self) {
        loop {
            // Check for messages
            let r = self.receiver.try_recv();
            match r {
                Ok(msg) => {
                    match msg {
                        messages::ReaderMsg::Terminate => break,
                        messages::ReaderMsg::StartListening => {
                            self.listen = true;
                            println!("Listening for UDP data...");
                        }
                        messages::ReaderMsg::StopListening => {
                            self.listen = false;
                            println!("Stopped listening UDP for data");
                        }
                    };
                },
                // Do nothing if there are no message matches
                _ => (),
            };
            // Are we in listen mode
            if self.listen {
                // Wait for UDP data or timeout so we can check the channel
                let r = self.p_sock.recv_from(self.udp_frame.as_mut());
                match r {
                    Ok((sz,_addr)) => {
                        //println!("Received {:?} data bytes", sz);
                        if sz == common_defs::FRAME_SZ as usize {
                            self.split_frame();
                        } else {
                            println!("Received incomplete frame {}, discarding!", sz);
                        }
                    }
                    Err(_e) => (), //println!("Error or timeout on receive data [{}]", e),
                } 
            }
        }
    }

    // Split frame into protocol fields and data content and decode
    fn split_frame(&mut self) { 
        
        let num_rx = globals::get_num_rx();
        let sel_rx = globals::get_sel_rx();
        let mut j: usize = 0;
        let mut ep6_seq : [u8; 4] = [0,0,0,0];
        
        // Unsafe because of potentially uninitialised array
        unsafe { 
            // Check for frame type
            if self.udp_frame[3].assume_init() == common_defs::EP6 {
                // We have a frame of IQ data
                // First 8 bytes are the header, then 2x512 bytes of data
                // The sync and cc bytes are the start of each data frame
                //
                // Extract and check the sequence number
                //  2    1   1   4
                // Sync Cmd End Seq
                // if the sequence number check fails it means we have missed some frames
                // Nothing we can do so it just gets reported.

                // Move sequence data into temp array
                for i in 4..8 {
                    ep6_seq[j] = (self.udp_frame[i as usize]).assume_init();
                    j += 1;
                }
                if !self.i_seq.check_ep6_seq(ep6_seq) {
                    //Boolean return incase we need to do anything
                    // Sequence errors are reported in cc-in
                }
            } else if self.udp_frame[3].assume_init() == common_defs::EP4 {
                // We have wideband data
                // TBD
                return;
            }
        }
        
        // Decode into contiguous IQ and Mic frames
        let num_smpls = protocol::decoder::frame_decode(
            num_rx, sel_rx, globals::get_smpl_rate(), 
            &self.udp_frame, &mut self.iq, &mut self.mic);

        //================================================================================
        // At this point we have separated the IQ and Mic data into separate buffers
        // Truncate vec if necessary for RX samples for current number of receivers
        let mut success = false;
        let mut vec_iq = self.iq.to_vec();
        if num_rx > 1 {
            vec_iq.resize((num_smpls*common_defs::BYTES_PER_SAMPLE) as usize, 0);
        }
        // Copy the UDP frame into the rb_iq ring buffer
        let r = self.rb_iq.write().write(&vec_iq);
        match r {
            Err(e) => {
                println!("Write error on rb_iq, skipping block {:?}", e);
            }
            Ok(_sz) => {
                success = true;  
            }
        }
        // Signal the pipeline that data is available
        if success {
            let mut locked = self.iq_cond.0.lock().unwrap();
            *locked = true;
            self.iq_cond.1.notify_one();
        } 
    }
}


//==================================================================================
// Thread startup
pub fn reader_start(
    receiver : crossbeam_channel::Receiver<messages::ReaderMsg>, 
    p_sock : Arc<socket2::Socket>, 
    rb_iq : Arc<ringb::SyncByteRingBuf>, 
    iq_cond : Arc<(Mutex<bool>, Condvar)>) -> thread::JoinHandle<()> {
    let join_handle = thread::spawn(  move || {
        reader_run(receiver, p_sock, rb_iq, iq_cond);
    });
    return join_handle;
}

fn reader_run(
    receiver : crossbeam_channel::Receiver<messages::ReaderMsg>, 
    p_sock : Arc<socket2::Socket>, 
    rb_iq : Arc<ringb::SyncByteRingBuf>,
    iq_cond : Arc<(Mutex<bool>, Condvar)>) {
    println!("UDP Reader running");

    // Instantiate the runtime object
    let mut i_reader = UDPRData::new(receiver,  p_sock, rb_iq, iq_cond);

    // Exits when the reader loop exits
    i_reader.reader_run();

    println!("UDP Reader exiting");
    thread::sleep(Duration::from_millis(1000));
}

