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
    prot_frame : [u8; common_defs::PROT_SZ as usize *2],
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
            prot_frame: [0; common_defs::PROT_SZ as usize *2],
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
        
        // Assume 1 radio for now
        let num_rx = globals::get_num_rx();
        let sel_rx = globals::get_sel_rx();
        let mut j: usize = 0;
        let mut ep6_seq : [u8; 4] = [0,0,0,0];
        let mut end_frame_1 = common_defs::END_FRAME_1;
        let mut end_frame_2 = common_defs::END_FRAME_2;
        let mut data_sz = common_defs::PROT_SZ * 2;
        let mut num_smpls = common_defs::NUM_SMPLS_1_RADIO;

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

                // For 1,2 radios the entire dataframe is used
                // For 3 radios there are 4 padding bytes in each frame
                // TBD: For now fix num_rx at one as we don't have the data yet 
                if num_rx == 2 {
                    num_smpls = common_defs::NUM_SMPLS_2_RADIO;
                } else if num_rx == 3 {
                    num_smpls = common_defs::NUM_SMPLS_3_RADIO;
                    end_frame_1 -= 4;
                    end_frame_2 -= 4;
                    data_sz -= 8;
                }

                // Extract the data from the UDP frame into the protocol frame
                // Select the correct RX data at this point
                // One RX   - I2(1)I1(1)10(1)Q2(1)Q1(1)Q0(1)MM etc
                // Two RX   - I2(1)I1(1)I0(1)Q2(1)Q1(1)Q0(1)I2(2)I1(2)I0(2)Q2(2)Q1(2)Q0(2)MM etc
                // Three RX - I2(1)I1(1)I0(1)Q2(1)Q1(1)Q0(1)I2(2)I1(2)I0(2)Q2(2)Q1(2)Q0(2)I2(3)I1(3)I0(3)Q2(3)Q1(3)Q0(3)MM etc
                //
                // So for one RX we take all the IQ data always. 
                // This is 63 samples of I/Q and 63 samples of Mic as 504/8 = 63.
                //
                // For 2 RX we take either just the RX1 or RX2 data depending on the selected receiver.
                // This is 36 samples of RX1, RX2 and Mic as 504/14 = 36
                //
                // For 3 RX we take RX1, RX2 or RX3 data depending on the selected receiver.
                // This is 25 samples of RX1, RX2, RX3 and Mic but 504/25 is 20 rm 4 so there are 4 nulls at the end.
                //

                /*
                // Frame 1
                j = 0;
                for b in common_defs::START_FRAME_1..=end_frame_1 {
                        self.prot_frame[j] = self.udp_frame[b as usize].assume_init();
                        j += 1;
                }
                // Frame 2
                for b in common_defs::START_FRAME_2..=end_frame_2 {
                    self.prot_frame[j] = self.udp_frame[b as usize].assume_init();
                    j += 1;
                }
                */

                // Tiny state machine, IQ, Mic. Skip
                const IQ:i32 = 0;
                const M:i32 = 1;
                const S1:i32 = 2;
                const S2:i32 = 3;
                let mut p_idx = 0;
                let mut smpls = common_defs::NUM_SMPLS_1_RADIO/2;

                if num_rx == 1 {
                    /* 
                    // Take all data as RX 1
                    // Frame 1
                    p_idx = 0;
                    for b in common_defs::START_FRAME_1..=end_frame_1 {
                            self.prot_frame[p_idx] = self.udp_frame[b as usize].assume_init();
                            p_idx += 1;
                    }
                    // Frame 2
                    for b in common_defs::START_FRAME_2..=end_frame_2 {
                        self.prot_frame[p_idx] = self.udp_frame[b as usize].assume_init();
                        p_idx += 1;
                    }
                    */
                    // Take all data
                    p_idx = 0;
                    for frame in 1..=2 {
                        smpls = common_defs::NUM_SMPLS_1_RADIO/2;
                        let mut state = IQ;
                        let mut index = common_defs::START_FRAME_1;
                        if frame == 2 {index = common_defs::START_FRAME_2};
                        for _smpl in 0..smpls*2 {
                            if state == IQ {
                                // Take IQ bytes
                                for b in index..index+common_defs::BYTES_PER_SAMPLE{
                                    self.prot_frame[p_idx] = self.udp_frame[b as usize].assume_init();
                                    p_idx += 1;
                                }
                                state = M;
                                index += common_defs::BYTES_PER_SAMPLE;
                            } else if state == M {
                                // Take Mic bytes
                                for b in index..index+common_defs::MIC_BYTES_PER_SAMPLE{
                                    self.prot_frame[p_idx] = self.udp_frame[b as usize].assume_init();
                                    p_idx += 1;
                                }
                                state = IQ;
                                index += common_defs::MIC_BYTES_PER_SAMPLE;
                            }
                        }
                    }
                } else if num_rx == 2 {
                    // Skip either RX 1 or RX 2 data
                    p_idx = 0;
                    for frame in 1..=2 {
                        smpls = common_defs::NUM_SMPLS_2_RADIO/2;
                        let mut state = IQ;
                        if sel_rx == 2 {state = S1};
                        let mut index = common_defs::START_FRAME_1;
                        if frame == 2 {index = common_defs::START_FRAME_2};
                        for _smpl in 0..smpls*3 {
                            if state == IQ {
                                // Take IQ bytes
                                for b in index..index+common_defs::BYTES_PER_SAMPLE{
                                    self.prot_frame[p_idx] = self.udp_frame[b as usize].assume_init();
                                    p_idx += 1;
                                }
                                if sel_rx == 1 {state = S1} else {state = M};
                                index += common_defs::BYTES_PER_SAMPLE;
                            } else if state == S1 {
                                // Skip IQ bytes
                                index += common_defs::BYTES_PER_SAMPLE;
                                if sel_rx == 1 {state = M} else {state = IQ};
                            } else if state == M {
                                // Take Mic bytes
                                for b in index..index+common_defs::MIC_BYTES_PER_SAMPLE{
                                    self.prot_frame[p_idx] = self.udp_frame[b as usize].assume_init();
                                    p_idx += 1;
                                }
                                if sel_rx == 1 {state = IQ} else {state = S1};
                                index += common_defs::MIC_BYTES_PER_SAMPLE;
                            }
                        }
                    }
                } else if num_rx == 3 {
                    // Skip RX 1, Rx 2 or RX 3 data
                    p_idx = 0;
                    for frame in 1..=2 {
                        smpls = common_defs::NUM_SMPLS_3_RADIO/2;
                        let mut state = IQ;
                        if sel_rx == 2 || sel_rx == 3 {state = S1};
                        let mut index = common_defs::START_FRAME_1;
                        if frame == 2 {index = common_defs::START_FRAME_2};
                        for _smpl in 0..smpls*4 {
                            if state == IQ {
                                // Take IQ bytes
                                for b in index..index+common_defs::BYTES_PER_SAMPLE{
                                    self.prot_frame[j] = self.udp_frame[b as usize].assume_init();
                                    p_idx += 1;
                                }
                                if sel_rx == 1 {state = S1} else if sel_rx == 2 {state = S2} else {state = M};
                                index += common_defs::BYTES_PER_SAMPLE;
                            } else if state == S1 {
                                // Skip IQ bytes
                                index = index + common_defs::BYTES_PER_SAMPLE;
                                if sel_rx == 1 {state = S2} else if sel_rx == 2 {state = IQ} else {state = S2};
                            } else if state == S2 {
                                // Skip IQ bytes
                                index = index + common_defs::BYTES_PER_SAMPLE;
                                if sel_rx == 1 {state = M} else if sel_rx == 2 {state = M} else {state = IQ};
                            } else if state == M {
                                // Take Mic bytes
                                for b in index..index+common_defs::MIC_BYTES_PER_SAMPLE{
                                    self.prot_frame[p_idx] = self.udp_frame[b as usize].assume_init();
                                    p_idx += 1;
                                }
                                if sel_rx == 1 {state = IQ} else if sel_rx == 2 {state = S1} else {state = S1};
                                index += common_defs::MIC_BYTES_PER_SAMPLE;
                            }
                        }
                    }
                }

            } else if self.udp_frame[3].assume_init() == common_defs::EP4 {
                // We have wideband data
                // TBD
                return;
            }
        }
        
        // We now have contiguous IQ data and Mic data from both protocol frames in prot_frame
        // Now decode the frame
        protocol::decoder::frame_decode(
            num_smpls, num_rx, globals::get_smpl_rate(), data_sz, 
                self.prot_frame, &mut self.iq, &mut self.mic);

        //================================================================================
        // At this point we have separated the IQ and Mic data into separate buffers
        // Copy the UDP frame into the rb_iq ring buffer
        // Truncate vec if necessary for RX samples for current RX
        let mut success = false;
        let mut vec_iq = self.iq.to_vec();
        if num_rx > 1 {
            vec_iq.resize((num_smpls*common_defs::BYTES_PER_SAMPLE) as usize, 0);
        }
        
        let r = self.rb_iq.write().write(&vec_iq);
        
        match r {
            Err(e) => {
                println!("Write error on rb_iq, skipping block {:?}", e);
            }
            Ok(_sz) => {
                success = true;  
            }
        }

        if success {
            // Signal the pipeline that data is available
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

