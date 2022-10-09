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

use std::ffi:: {CString, c_int, c_double, c_long};
use std::os::raw::c_char;

use crate::app::common::common_defs;

// External interfaces exposed through the WDSP library
#[link(name = "wdsp")]
extern "C" {
	fn WDSPwisdom(s: *const c_char);
	fn OpenChannel(
		ch_id: i32, in_sz: i32, dsp_sz: i32, 
		in_rate: i32, dsp_rate: i32, out_rate: i32, 
		ch_type: i32, state: i32, tdelayup: f64, 
		tslewup: f64, tdelaydown: f64, tslewdown: f64, bfo: i32);
}

// Run WDSP wisdom to optimise the FFT sizes
// This is always called at start of day and will do nothing if the file exists
pub fn wdsp_wisdom() {
	let s  = CString::new("./").unwrap();
    unsafe {WDSPwisdom(s.as_ptr())};
}

// Open WDSP channel
pub fn wdsp_open_ch(
		ch_type:i32, ch_id: i32, iq_sz: i32, mic_sz: i32, 
		in_rate: i32, out_rate: i32, tdelayup: f64, 
		tslewup: f64, tdelaydown:f64, tslewdown:f64) {
	/* Open a new DSP channel
	**
	** Arguments:
	** 	ch_type 	-- CH_RX | CH_TX
	**	channel		-- Channel to use
	** 	iq_size		-- 128, 256, 1024, 2048, 4096
	** 	mic_size	-- as iq_size for same sample rate
	** 	in_rate		-- input sample rate
	** 	out_rate	-- output sample rate
	** 	tdelayup	-- delay before up slew
	** 	tslewup		-- length of up slew
	**  tdelaydown	-- delay before down slew
	** 	tslewdown	-- length of down slew
	**
	** Note:
	** 	There are additional parameters to open_channel. These are handled as follows:
	** 		o in_size - the number of samples supplied to the channel.
	** 		o input_samplerate - taken from the set_speed() API call, default 48K.
	** 		o dsp_rate - same as input_samplerate.
	** 		o output_samplerate - fixed at 48K for RX TBD TX
	**
	** The channel is not automatically started. Call set_ch_state() to start the channel.
	*/


	let mut input_sz: i32;
	let mut dsp_rate: i32;

	if ch_type == common_defs::CH_RX as i32 {
		// For RX we keep the input and dsp size the same.
		input_sz = iq_sz;
	} else {
		// For TX we arrange that the same number of samples arrive at the output as for RX
		// This depends on the input and output rates
		input_sz = mic_sz;
	}
	// Set the internal rate to the input samplerate
	dsp_rate = in_rate;

	// Open the channel
	// There is no return value so will probably crash if there is a problem
	unsafe{OpenChannel(
		ch_id, input_sz, input_sz, 
		in_rate, dsp_rate, out_rate, 
		ch_type, common_defs::STATE_STOPPED as i32, 
		tdelayup, tslewup, tdelaydown, tslewdown, 1)};

}

// Close WDSP channels
pub fn wdsp_close_ch() {
	
}
