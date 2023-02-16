/*
decoder.rs

Module - decoder
Module decoder manages decoding the protocol frame

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

use std::mem::MaybeUninit;

use crate::app::common::common_defs;

// Decode the IQ frame
pub fn frame_decode(
		num_rx: u32, sel_rx: u32, rate: u32,
		udp_frame : &[MaybeUninit<u8>; common_defs::FRAME_SZ as usize],
		iq: &mut [u8; common_defs::IQ_ARR_SZ_R1 as usize],
		mic: &mut [u8; common_defs::MIC_ARR_SZ_R1 as usize]) -> u32 {

	// Extract the data from the UDP frame into the IQ and Mic frames
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
	// For 48KHz sample rate we take all Mic samples
	// For 96KHz sample rate we take every second sample
	// For 192KHz sample rate we take every fourth sample

	// Tiny state machine, IQ, Mic. Skip
	const IQ:i32 = 0;
	const M:i32 = 1;
	const SIQ1:i32 = 2;
	const SIQ2:i32 = 3;
	const SM1:i32 = 4;
	const SM2:i32 = 5;
	const SM3:i32 = 6;

	// Index into IQ output data
	let mut idx_iq;
	// Index into Mic output data
	let mut idx_mic;
	// Number of samples of IQ and Mic for one receiver in one UDP frame
	let mut smpls = common_defs::NUM_SMPLS_1_RADIO/2;

	if num_rx == 1 {
		// Take all I/Q and Mic data for one receiver
		idx_iq = 0;
		idx_mic = 0;
		for frame in 1..=2 {
			let mut state = IQ;
			let mut index = common_defs::START_FRAME_1;
			if frame == 2 {index = common_defs::START_FRAME_2};
			for _smpl in 0..smpls*2 {
				if state == IQ {
					// Take IQ bytes
					for b in index..index+common_defs::BYTES_PER_SAMPLE{
						iq[idx_iq] = unsafe{udp_frame[b as usize].assume_init()};
						idx_iq += 1;
					}
					state = M;
					index += common_defs::BYTES_PER_SAMPLE;
				} else if state == M {
					// Take Mic bytes
					for b in index..index+common_defs::MIC_BYTES_PER_SAMPLE{
						mic[idx_mic] = unsafe{udp_frame[b as usize].assume_init()};
						idx_mic += 1;
					}
					state = IQ;
					index += common_defs::MIC_BYTES_PER_SAMPLE;
				}
			}
		}
	} else if num_rx == 2 {
		// Skip either RX 1 or RX 2 data
		idx_iq = 0;
		idx_mic = 0;
		for frame in 1..=2 {
			smpls = common_defs::NUM_SMPLS_2_RADIO/2;
			let mut state = IQ;
			if sel_rx == 2 {state = SIQ1};
			let mut sub_state;
			if rate == common_defs::SMPLS_48K {sub_state = M;} else {sub_state = SM1;}
			let mut index = common_defs::START_FRAME_1;
			if frame == 2 {index = common_defs::START_FRAME_2;};
			for _smpl in 0..smpls*3 {
				if state == IQ {
					// Take IQ bytes
					for b in index..index+common_defs::BYTES_PER_SAMPLE{
						iq[idx_iq] = unsafe{udp_frame[b as usize].assume_init()};
						idx_iq += 1;
					}
					if sel_rx == 1 {state = SIQ1} else {state = M};
					if rate == common_defs::SMPLS_48K {sub_state = M;} else {sub_state = SM1;}
					index += common_defs::BYTES_PER_SAMPLE;
				} else if state == SIQ1 {
					// Skip IQ bytes
					index += common_defs::BYTES_PER_SAMPLE;
					if sel_rx == 1 {state = M} else {state = IQ};
					if rate == common_defs::SMPLS_48K {sub_state = M;} else {sub_state = SM1;}
				} else if state == M {
					// Skip 1,2 or 3 samples if > 48KHz
					if sub_state == SM1 {
						index += common_defs::MIC_BYTES_PER_SAMPLE;
						if rate == common_defs::SMPLS_192K {
							sub_state = SM2;
						} else {
							sub_state = M;
						} 
					} else if sub_state == SM2 {
						index += common_defs::MIC_BYTES_PER_SAMPLE;
						if rate == common_defs::SMPLS_192K {
							sub_state = SM3;
						} else {
							sub_state = M;
						} 
					} else if sub_state == SM3 {
						index += common_defs::MIC_BYTES_PER_SAMPLE;
						sub_state = M;
					} else {
						for b in index..index+common_defs::MIC_BYTES_PER_SAMPLE{
							mic[idx_mic] = unsafe{udp_frame[b as usize].assume_init()};
							idx_mic += 1;
						}
						index += common_defs::MIC_BYTES_PER_SAMPLE;
					}
					if sel_rx == 1 {state = IQ} else {state = SIQ1};
				}
			}
		}
	} else if num_rx == 3 {
		// Skip RX 1, Rx 2 or RX 3 data
		idx_iq = 0;
		idx_mic = 0;
		for frame in 1..=2 {
			smpls = common_defs::NUM_SMPLS_3_RADIO/2;
			let mut state = IQ;
			if sel_rx == 2 || sel_rx == 3 {state = SIQ1};
			let mut sub_state;
			if rate == common_defs::SMPLS_48K {sub_state = M;} else {sub_state = SM1;}
			let mut index = common_defs::START_FRAME_1;
			if frame == 2 {index = common_defs::START_FRAME_2};
			for _smpl in 0..smpls*4 {
				if state == IQ {
					// Take IQ bytes
					for b in index..index+common_defs::BYTES_PER_SAMPLE{
						iq[idx_iq] = unsafe{udp_frame[b as usize].assume_init()};
						idx_iq += 1;
					}
					if sel_rx == 1 {state = SIQ1} else if sel_rx == 2 {state = SIQ2} else {state = M};
					if rate == common_defs::SMPLS_48K {sub_state = M;} else {sub_state = SM1;}
					index += common_defs::BYTES_PER_SAMPLE;
				} else if state == SIQ1 {
					// Skip IQ bytes
					index = index + common_defs::BYTES_PER_SAMPLE;
					if sel_rx == 1 {state = SIQ2} else if sel_rx == 2 {state = IQ} else {state = SIQ2};
					if rate == common_defs::SMPLS_48K {sub_state = M;} else {sub_state = SM1;}
				} else if state == SIQ2 {
					// Skip IQ bytes
					index = index + common_defs::BYTES_PER_SAMPLE;
					if sel_rx == 1 {state = M} else if sel_rx == 2 {state = M} else {state = IQ};
					if rate == common_defs::SMPLS_48K {sub_state = M;} else {sub_state = SM1;}
				} else if state == M {
					// Skip 1,2 or 3 samples if > 48KHz
					if sub_state == SM1 {
						index += common_defs::MIC_BYTES_PER_SAMPLE;
						if rate == common_defs::SMPLS_192K {
							sub_state = SM2;
						} else {
							sub_state = M;
						} 
					} else if sub_state == SM2 {
						index += common_defs::MIC_BYTES_PER_SAMPLE;
						if rate == common_defs::SMPLS_192K {
							sub_state = SM3;
						} else {
							sub_state = M;
						} 
					} else if sub_state == SM3 {
						index += common_defs::MIC_BYTES_PER_SAMPLE;
						sub_state = M;
					} else {
						for b in index..index+common_defs::MIC_BYTES_PER_SAMPLE{
							mic[idx_mic] = unsafe{udp_frame[b as usize].assume_init()};
							idx_mic += 1;
						}
						index += common_defs::MIC_BYTES_PER_SAMPLE;
					}
					if sel_rx == 1 {state = IQ} else {state = SIQ1};
				}
			}
		}
	}
	// Return total number of samples transferred
	return smpls*2;
}
