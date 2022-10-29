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
use std::ops::Neg;

use crate::app::common::common_defs;

static mut g_mode: i32 = 0;
static mut g_filter: i32 = 0;

// External interfaces exposed through the WDSP library
#[link(name = "wdsp_win")]
extern "C" {
	fn WDSPwisdom(s: *const c_char);
	fn OpenChannel(
		ch_id: i32, in_sz: i32, dsp_sz: i32, 
		in_rate: i32, dsp_rate: i32, out_rate: i32, 
		ch_type: i32, state: i32, tdelayup: f64, 
		tslewup: f64, tdelaydown: f64, tslewdown: f64);
	fn SetChannelState (ch_id: i32, state: i32, dmode: i32) -> i32;
	fn fexchange0(ch_id: i32, in_buf: *mut f64, out_buf: *mut f64, error: *mut i32);
	fn SetRXAMode(ch_id: i32, mode: i32);
	fn SetRXABandpassRun(ch_id: i32, run: i32);
	fn SetRXABandpassFreqs(ch_id: i32, low: f64, high: f64);
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
		tdelayup, tslewup, tdelaydown, tslewdown)};

}

// Set channel state
pub fn wdsp_set_ch_state(ch_id: i32, state: i32, dmode: i32) -> i32 {
	unsafe{return SetChannelState(ch_id, state, dmode)}
}

// Close WDSP channels
pub fn wdsp_close_ch() {
	
}

// Data exchange
pub fn wdsp_exchange(ch_id: i32, in_buf: &mut [f64; (common_defs::DSP_BLK_SZ * 2) as usize],  out_buf: &mut [f64; (common_defs::DSP_BLK_SZ * 2) as usize], error: &mut i32) {	
	unsafe{fexchange0(ch_id,  in_buf.as_mut_ptr(),  out_buf.as_mut_ptr(),  error)}	
}

// Modes and filters
pub fn wdsp_set_rx_mode(ch_id: i32, mode: i32) {
	unsafe{g_mode = mode;}
	set_mode_filter(ch_id);
}

pub fn wdsp_set_rx_filter(ch_id: i32, filter: i32) {
	// Filters are 0-7 in order
	// 6K 4K 2.7K 2.4K 1.0K 500Hz 250Hz 100Hz
	unsafe{g_filter = filter;}
	set_mode_filter(ch_id);
}

fn set_mode_filter(ch_id: i32) {
	let mut low: i32 = 0;
	let mut high: i32 = 0;
	unsafe{
		let filter = g_filter;
		let mode = g_mode;
		let mut new_low = 0;
		let mut new_high = 0;
		match filter {
			0 => {low = 100; high = 6100},
			1 => {low = 100; high = 4100},
			2 => {low = 300; high = 3000},
			3 => {low = 300; high = 2700},
			4 => {low = 300; high = 1300},
			5 => {low = 500; high = 1000},
			6 => {low = 600; high = 850},
			7 => {low = 700; high = 800},
			_ => (),
		}
		if mode == 0 || mode == 3 || mode == 9 {
			// Low sideband so move filter to low side
			new_low = high.neg();
			new_high = low.neg();
		} else if mode == 1 || mode == 4 || mode == 7 {
			// High sideband so leave
			new_low = low;
			new_high = high;
		} else {
			// Both sidebands required
			new_low = high.neg();
			new_high = high;
		}
	
		SetRXAMode(ch_id, mode);
		SetRXABandpassRun(ch_id, 1);
		SetRXABandpassFreqs(ch_id, new_low as f64, new_high as f64);
	}
}
