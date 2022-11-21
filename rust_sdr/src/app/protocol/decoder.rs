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

use crate::app::common::common_defs;

// Decode the IQ frame
pub fn frame_decode(
		n_smpls: u32, n_rx: u32, rate: u32, in_sz: u32,
	 	udp_frame: [u8; common_defs::PROT_SZ as usize * 2],
		iq: &mut [u8; common_defs::IQ_ARR_SZ_R1 as usize],
		mic: &mut [u8; common_defs::MIC_ARR_SZ_R1 as usize]) {

	/* Decode the incoming data packet
	*
	* Arguments:
	*  	n_smpls				--	number of I/Q samples per frame per receiver
	*  	n_rx				--	number of receivers
	*  	rate				-- 	48000/96000/192000
	* 	in_sz				--	size of input data buffer
	*  	udp_frame   		--  input data
	*	iq					--	IQ output data
	*	mic					--	Mic output data
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

	// Current state
	const IQ: u32 = 0;
	const MIC: u32 = 1;

	// Depends on relative rate
	let mic_blk_sel = rate / 48000;
	// This is a count of blocks to skip and must be static
	static mut _SKIP_MIC_DATA: i32 = 0;

	// Reset the peak input level
	let _sample_input_level: i16 = 0;
	let _peak_input_inst: i16 = 0;

	// The total number of IQ bytes to be concatenated
	let total_iq_bytes: u32 = n_smpls * n_rx * 6;	// 6 bytes per sample (2 x 24 bit)
	let mut total_iq_bytes_ct: u32 = total_iq_bytes;		// iteration counter

	// Determine if we are using HPSDR or local mic input
	// Note that for local we let the normal processing run through except when it comes to
	// writing to the ring buffer we write data from the local audio input ring buffer to
	// the mic ring buffer.
	// TBD
	let mut _local = false;
	
	// The total number of Mic bytes to be moved
	let total_mic_bytes: u32;
	if mic_blk_sel == 1 {
		// Take every byte in every frame
		total_mic_bytes = n_smpls * 2;	// 2 bytes per sample (1 x 16 bit)
	}
	else {
		// Take one frame and leave the other frame
		// We then skip frames if necessary
		total_mic_bytes = n_smpls;
	}
	let mut total_mic_bytes_ct: u32 = total_mic_bytes;

	// The number of IQ bytes for each receiver(s) sample
	let iq_bytes = n_rx * common_defs::BYTES_PER_SAMPLE;
	// IQ byte counter
	let mut iq_ct = iq_bytes;
	// The number of Mic bytes following receiver sample
	let mic_bytes = 2;
	// Mic byte counter
	let mut mic_ct = mic_bytes;
	// Initial state is reading IQ bytes as we always align at the start of IQ data
	let mut state = IQ;

	// Iterate through every input byte
	let mut iq_index = 0;
	let mut mic_index = 0;
	for i in 0..in_sz {
		if state == IQ {
			// Processing IQ bytes
			if total_iq_bytes_ct > 0 {
				iq[iq_index] = udp_frame[i as usize];
				iq_index += 1;
				total_iq_bytes_ct -= 1;
			}
			iq_ct -= 1;
			if iq_ct == 0 {
				// Exhausted bytes for receiver(s) sample
				// Set the Mic count and change state
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
			mic_ct -= 1;
			if mic_ct == 0 {
				// Exhausted bytes for receiver(s) sample
				// Set the Mic count and change state
				iq_ct = iq_bytes;
				state = IQ;
			}
		}
	}

	// We have now extracted contiguous IQ and Mic samples into separate buffers
	
	// Process Mic data and local
	// TBD
}