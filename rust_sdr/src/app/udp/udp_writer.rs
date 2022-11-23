/*
udp_writer.rs

Module - udp_writer
Manages write data over UDP to the SDR hardware

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
use socket2;
use std::sync::{Arc, Mutex, Condvar};
use std::io:: Read;

use crate::app::common::common_defs;
use crate::app::common::messages;
use crate::app::protocol;
use crate::app::common::ringb;

#[allow(dead_code)]
pub struct UDPWData{
    receiver : crossbeam_channel::Receiver<messages::WriterMsg>,
    p_sock : Arc<socket2::Socket>,
    p_addr : Arc<socket2::SockAddr>,
    rb_audio : Arc<ringb::SyncByteRingBuf>,
    audio_cond : Arc<(Mutex<bool>, Condvar)>,
    udp_frame : [u8; common_defs::FRAME_SZ as usize],
    prot_frame : [u8; common_defs::PROT_SZ as usize*2],
    pub i_cc : Arc<Mutex<protocol::cc_out::CCData>>,
    pub i_seq: protocol::seq_out::SeqData,
    listen : bool,
}

// Implementation methods on CCData
//impl UDPWData<'_> {
impl UDPWData {
	// Create a new instance and initialise the default arrays
	pub fn new(
            receiver : crossbeam_channel::Receiver<messages::WriterMsg>,
            p_sock : Arc<socket2::Socket>, 
            p_addr : Arc<socket2::SockAddr>,
            rb_audio : Arc<ringb::SyncByteRingBuf>,
            audio_cond : Arc<(Mutex<bool>, Condvar)>,
            i_cc : Arc<Mutex<protocol::cc_out::CCData>>) -> UDPWData {
        // Initialise the cc data
        i_cc.lock().unwrap().cc_init();
        // Create an instance of the sequence type
        let i_seq = protocol::seq_out::SeqData::new();

		UDPWData {
            receiver: receiver,
			p_sock: p_sock,
            p_addr: p_addr,
            rb_audio: rb_audio,
            audio_cond: audio_cond,
            udp_frame: [0; common_defs::FRAME_SZ as usize],
            prot_frame: [0; common_defs::PROT_SZ as usize *2],
            i_cc : i_cc,
            i_seq: i_seq,
            listen: false,
		}
	}

    // This is the thread main loop. When this exits the thread exits.
    pub fn writer_run(&mut self) {
        loop {
            // Check for messages
            let r = self.receiver.try_recv();
            match r {
                Ok(msg) => {
                    match msg {
                        messages::WriterMsg::Terminate => break,
                        messages::WriterMsg::PrimeHardware => {
                            self.prime();
                        }
                        messages::WriterMsg::WriteData => {
                            self.write_data();
                        }
                    };
                },
                // Do nothing if there are no message matches
                _ => (),
            };
            // Send any outgoing data
            self.write_data();

            thread::sleep(Duration::from_millis(10));
        }
    }

    // Send a fully set of cc bytes to prime the radio before starting to listen
    pub fn prime(&mut self) {
        
        for _i in 0..6 {
            // Encode the next frame
            protocol::encoder::encode(&mut self.i_seq, &mut self.i_cc.lock().unwrap(), &mut self.udp_frame, &mut self.prot_frame);
            // Send to hardware
            let r = self.p_sock.send_to(&self.udp_frame, &self.p_addr);
            match r {
                Ok(_sz) => (),
                Err(e) => println!("Error sending [{}]", e),
            } 
        }
        println!("Sent prime data for all cc values");
    }

    pub fn write_data(&mut self) {
        loop {
            // Enough data available
            let r = self.rb_audio.try_read();   
            match r {
                Ok(mut m) => {
                    let prot_frame = m.read(&mut self.prot_frame);
                    match prot_frame {
                        Ok(_sz) => {
                            // Encode the next frame
                            protocol::encoder::encode(&mut self.i_seq, &mut self.i_cc.lock().unwrap(), &mut self.udp_frame, &mut self.prot_frame);
                            // Send to hardware
                            let r = self.p_sock.send_to(&self.udp_frame, &self.p_addr);
                            match r {
                                Ok(_sz) => (),
                                Err(e) => println!("Error sending [{}]", e),
                            } 
                        }
                        Err(_e) => {
                            // Not enough data available so try next time
                            break;
                        }
                    }
                }
                Err(_e) => {
                    // Couldn't get lock so try next time
                    break;
                }
            }
        }
    }
}

//==================================================================================
// Thread startup
pub fn writer_start(
        receiver : crossbeam_channel::Receiver<messages::WriterMsg>, 
        p_sock : Arc<socket2::Socket>,
        p_addr : Arc<socket2::SockAddr>, 
        rb_audio : Arc<ringb::SyncByteRingBuf>, 
        audio_cond : Arc<(Mutex<bool>, Condvar)>,
        i_cc : Arc<Mutex<protocol::cc_out::CCData>>) -> thread::JoinHandle<()> {
    let join_handle = thread::spawn(  move || {
        writer_run(receiver, p_sock, p_addr, rb_audio, audio_cond, i_cc);
    });
    return join_handle;
}

fn writer_run(
    receiver : crossbeam_channel::Receiver<messages::WriterMsg>, 
    p_sock : Arc<socket2::Socket>,
    p_addr : Arc<socket2::SockAddr>, 
    rb_audio : Arc<ringb::SyncByteRingBuf>,
    audio_cond : Arc<(Mutex<bool>, Condvar)>,
    i_cc : Arc<Mutex<protocol::cc_out::CCData>>) {
    println!("UDP Writer running");

    // Instantiate the runtime object
    let mut i_writer = UDPWData::new(receiver, p_sock, p_addr, rb_audio, audio_cond, i_cc);

    // Exits when the reader loop exits
    i_writer.writer_run();

    println!("UDP Writer exiting");
    thread::sleep(Duration::from_millis(1000));
}