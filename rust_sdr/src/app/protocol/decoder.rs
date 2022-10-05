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

use std::collections::VecDeque;

use crate::app::common::common_defs;

pub fn frame_decode(n_smpls: u32, n_rx: u32, rate: u32, in_sz: u32, udp_frame: [u8; common_defs::PROT_SZ as usize * 2]) {

	/* Decode the incoming data packet
	*
	* Arguments:
	*  	n_smpls				--	number of I/Q samples per frame per receiver
	*  	n_rx				--	number of receivers
	*  	rate				-- 	48000/96000/192000
	* 	in_sz				--	size of input data buffer
	*  	ptr_in_bytes   		--  ptr to the input data
	*/

	// Data exchange operates on the ring buffers.
	// 	if there is room to add the data the data is written, else the block is skipped
	//
	// The data is pre-processed such that only contiguous data is written to the ring buffers
	// separated into IQ and Mic data.

	// The Mic data is repeated at higher sampling rates
	// 48K = 1, 96K = 2, 192K = 4, 384K = 8
	// At 1 we take all blocks,
	// at 2 we take every 2nd block
	// at 4 we take every 4th block
	// at 8 we take every 8th block

	let deque: VecDeque<u8> = VecDeque::with_capacity(10000);

	// Current state
	const IQ: u32 = 0;
	const MIC: u32 = 1;

	let mic_blk_sel = rate / 48000;
	// This is a count of blocks to skip and must be static
	static skip_mic_data: i32 = 0;

	// Set of variables to manage the decode
	let mut nskip = 0;
	let mut total_iq_bytes: u32 = 0;
	let mut total_mic_bytes: u32 = 0;
	let mut total_iq_bytes_ct: u32 = 0;
	let mut total_mic_bytes_ct: u32 = 0;
	let mut iq_bytes = 0;
	let mut iq_ct = 0;
	let mut mic_bytes = 0;
	let mut mic_ct = 0;
	let mut iq_index = 0;
	let mut mic_index = 0;
	let mut state = 0;
	let mut signal = 0;
	let mut sample_input_level: i16 = 0;
	let mut peak_input_inst: i16 = 0;
	let mut i = 0;
	let mut local: bool = false;
	let mut ret = 0;
	let mut write_space = 0;
	let mut read_space  = 0;
	let mut xfer_sz = 0;

	// Reset the peak input level
	sample_input_level = 0;
	peak_input_inst = 0;

	// The total number of IQ bytes to be concatenated
	total_iq_bytes = n_smpls * n_rx * 6;	// 6 bytes per sample (2 x 24 bit)
	total_iq_bytes_ct = total_iq_bytes - 1;

	// Determine if we are using HPSDR or local mic input
	// Note that for local we let the normal processing run through except when it comes to
	// writing to the ring buffer we write data from the local audio input ring buffer to
	// the mic ring buffer.
	// TBD
	local = false;
	
	// The total number of Mic bytes to be moved
	if mic_blk_sel == 1 {
		// Take every byte in every frame
		total_mic_bytes = n_smpls * 2;	// 2 bytes per sample (1 x 16 bit)
	}
	else {
		// Take one frame and leave the other frame
		// We then skip frames if necessary
		total_mic_bytes = n_smpls;
	}
	total_mic_bytes_ct = total_mic_bytes - 1;

	// We need some buffers to move the IQ data into when we consolidate it.
	// The size of these buffers can vary per call. However Rust arrays are
	// fixed size so we create the maximum size we might need.
	let mut iq: [u8; common_defs::IQ_ARR_SZ as usize] = [0; common_defs::IQ_ARR_SZ as usize];
	// For the mic data we need two buffers, one to pack the incoming mic data
	// from the hardware and one for local mic data from the local device.
	let mut mic: [u8; common_defs::MIC_ARR_SZ as usize] = [0; common_defs::MIC_ARR_SZ as usize];
	//TBD

	// The number of IQ bytes for each receiver(s) sample
	iq_bytes = n_rx * common_defs::BYTES_PER_SAMPLE;
	// IQ byte counter
	iq_ct = iq_bytes;
	// The number of Mic bytes following receiver sample
	mic_bytes = 2;
	// Mic byte counter
	mic_ct = mic_bytes;
	// Initial state is reading IQ bytes as we always align at the start of IQ data
	state = IQ;

	// Iterate through every input byte
	iq_index = 0;
	mic_index = 0;
	for i in 0..in_sz {
		if state == IQ {
			// Processing IQ bytes
			if total_iq_bytes_ct > 0 {
				iq[iq_index] = udp_frame[i as usize];
				iq_index += 1;
				total_iq_bytes_ct -= 1;
			}
			if iq_ct - 1 == 0 {
				// Exhausted bytes for receiver(s) sample
				// Set the Mic count and change state
				iq_ct -= 1;
				mic_ct = mic_bytes;
				state = MIC;
			}
		} else if state == MIC {
			// Processing Mic bytes
			if total_mic_bytes_ct > 0 {
				mic[mic_index] = udp_frame[i as usize];
				mic_index += 1;
				total_mic_bytes_ct -= 1;
			}
			if mic_ct - 1 == 0 {
				// Exhausted bytes for receiver(s) sample
				// Set the Mic count and change state
				mic_ct -= 1;
				iq_ct = iq_bytes;
				state = IQ;
			}
		}
	}

	// We have now extracted contiguous IQ and Mic samples into separate buffers
	
	// Process Mic data and local
	// TBD
}