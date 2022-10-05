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
use std::option;
use std::mem::MaybeUninit;
use std::sync::Arc;
use std::io::{self, Write};

use socket2;

use crate::app::protocol;
use crate::app::common::ringb;
use crate::app::common::common_defs;
use crate::app::common::messages;

//==================================================================================
// Runtime object for thread
pub struct UDPRData<'a>{
    receiver : crossbeam_channel::Receiver<messages::ReaderMsg>,
	p_sock :  &'a socket2::Socket,
    rb_iq : &'a ringb::SyncByteRingBuf,
    udp_frame : [MaybeUninit<u8>; common_defs::FRAME_SZ as usize],
    prot_frame : [u8; common_defs::PROT_SZ as usize *2],
    //pub i_cc: protocol::cc_in::CCDataMutex,
    pub i_seq: protocol::seq_in::SeqData,
    listen: bool,
}

// Implementation methods on UDPRData
impl UDPRData<'_> {
	// Create a new instance and initialise the default arrays
    pub fn new<'a>(receiver : crossbeam_channel::Receiver<messages::ReaderMsg>, p_sock : &'a socket2::Socket, rb_iq : &'a ringb::SyncByteRingBuf) -> UDPRData<'a> {
        // Create an instance of the cc_in type
        //let i_cc = protocol::cc_in::CCDataMutex::new();
        // Create an instance of the sequence type
        let i_seq = protocol::seq_in::SeqData::new();

		UDPRData {
            receiver: receiver,
			p_sock: p_sock,
            rb_iq : rb_iq,
            udp_frame: [MaybeUninit::uninit(); common_defs::FRAME_SZ as usize],
            prot_frame: [0; common_defs::PROT_SZ as usize *2],
            //i_cc: i_cc,
            i_seq: i_seq,
            listen: false,
		}
	}

    // Run loop for reader
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
                            println!("Listening for data...");
                        }
                        messages::ReaderMsg::StopListening => {
                            self.listen = false;
                            println!("Stopped listening for data");
                        }
                    };
                },
                // Do nothing if there are no message matches
                _ => (),
            };
            // Check for read data
            if self.listen {
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
        //static mut count: u32 = 0;
        //static mut done: bool = false;
        let num_rx = 1;
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
                let mut j: usize = 0;
                let mut ep6_seq : [u8; 4] = [0,0,0,0];

                for i in 4..8 {
                    ep6_seq[j] = (self.udp_frame[i as usize]).assume_init();
                    j += 1;
                }
                if self.i_seq.check_ep6_seq(ep6_seq) {
                    //print!("*");
                    //io::stdout().flush().unwrap();
                    //count += 1;
                } else {
                    //print!("x");
                    //io::stdout().flush().unwrap();
                    //if !done {
                    //    println!("\nCount on seq error {}", count);
                    //    if count != 0 {done = true};
                    //}
                }

                // For 1,2 radios the entire dataframe is used
                // For 3 radios there are 4 padding bytes in each frame
                // TBD: For now fix num_rx at one as we don't have the data yet 
                let mut end_frame_1 = common_defs::END_FRAME_1;
                let mut end_frame_2 = common_defs::END_FRAME_2;
                let mut data_sz = common_defs::PROT_SZ * 2;
                let mut num_smpls = common_defs::NUM_SMPLS_1_RADIO;
                if num_rx == 2 {
                    num_smpls = common_defs::NUM_SMPLS_2_RADIO;
                } else if num_rx == 3 {
                    num_smpls = common_defs::NUM_SMPLS_3_RADIO;
                    end_frame_1 -= 4;
                    end_frame_2 -= 4;
                    data_sz -= 8;
                }

                // Extract the data from the UDP frame into the protocol frame
                j = 0;
                for b in common_defs::START_FRAME_1..end_frame_1 {
                    self.prot_frame[j] = self.udp_frame[b as usize].assume_init();
                    j += 1;
                }
                j = 0;
                for b in common_defs::START_FRAME_2..end_frame_2 {
                    self.prot_frame[j] = self.udp_frame[b as usize].assume_init();
                    j += 1;
                }
            } else if self.udp_frame[3].assume_init() == common_defs::EP4 {
                // We have wideband data
                // TBD
            }
        }
        protocol::decoder::frame_decode(common_defs::NUM_SMPLS_1_RADIO, num_rx, common_defs::SMPLS_48K, common_defs::FRAME_SZ*2, self.prot_frame);

        // Copy the UDP frame into the rb_iq ring buffer
        let vec_proc_frame = self.prot_frame.to_vec();
        let r = self.rb_iq.try_write();
        match r {
            Ok(mut m) => {
                let r = m.write(&vec_proc_frame);
                match r {
                    Err(e) => println!("Write error on rb_iq {:?}", e),
                    Ok(sz) => {
                        println!("Wrote {:?} bytes to rb_iq", sz);
                    }
                }
            }
            Err(e) => println!("Write lock error on rb_iq {:?}", e),
        }
            
    }

}


//==================================================================================
// Thread startup
pub fn reader_start(receiver : crossbeam_channel::Receiver<messages::ReaderMsg>, p_sock : Arc<socket2::Socket>, rb_iq : Arc<ringb::SyncByteRingBuf>) -> thread::JoinHandle<()>{
    let join_handle = thread::spawn(  move || {
        reader_run(receiver, &p_sock, &rb_iq);
    });
    return join_handle;
}

fn reader_run(receiver : crossbeam_channel::Receiver<messages::ReaderMsg>, p_sock : &socket2::Socket, rb_iq : &ringb::SyncByteRingBuf) {
    println!("UDP Reader running");

    // Instantiate the runtime object
    let mut i_reader = UDPRData::new(receiver,  p_sock, rb_iq);

    // Exits when the reader loop exits
    i_reader.reader_run();

    println!("UDP Reader exiting");
    thread::sleep(Duration::from_millis(1000));
}

