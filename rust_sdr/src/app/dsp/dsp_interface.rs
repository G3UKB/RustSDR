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

use crate::app::common::globals;

use std::ffi:: {CString};
use std::os::raw::c_char;
use std::ops::Neg;

use crate::app::common::common_defs;

// External interfaces exposed through the WDSP library
#[link(name = "wdsp_win")]
extern "C" {
	fn WDSPwisdom(s: *const c_char);

	fn OpenChannel(
		ch_id: i32, in_sz: i32, dsp_sz: i32, 
		in_rate: i32, dsp_rate: i32, out_rate: i32, 
		ch_type: i32, state: i32, tdelayup: f64, 
		tslewup: f64, tdelaydown: f64, tslewdown: f64);
	fn CloseChannel(disp_id: i32);
	fn SetChannelState (ch_id: i32, state: i32, dmode: i32) -> i32;
	fn SetInputSamplerate (ch_id: i32, in_rate: i32);
	fn SetDSPSamplerate (ch_id: i32, dsp_rate: i32);
	fn fexchange0(ch_id: i32, in_buf: *mut f64, out_buf: *mut f64, error: *mut i32);

	fn XCreateAnalyzer ( 
		disp_id: i32,
		success: *mut i32,
		m_size: i32,
		m_LO: i32,
		m_stitch: i32,
		app_data_path: *mut i8);
	fn SetAnalyzer ( 
		disp_id: i32,
		n_fft: i32,
		typ: i32,
		flp: *mut i32,
		fft_sz: i32,
		bf_sz: i32,
		win_type: i32,
		pi: f64,
		ovrlp: i32,
		clp: i32,
		fscLin: i32,
		fscHin: i32,
		n_pix: i32,
		n_stch: i32,
		av_m: i32,
		n_av: i32,
		av_b: f64,
		calset: i32,
		fmin: f64,
		fmax: f64,
		max_w: i32);
	fn DestroyAnalyzer(disp_id: i32);
	fn GetPixels(disp_id: i32, out_real: *mut f32, flag: *mut i32);
	fn Spectrum2(disp_id: i32, ss: i32, LO: i32, in_real: *mut f32);

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

	let input_sz: i32;
	let dsp_rate: i32;

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
pub fn wdsp_close_ch(ch_id: i32) {
	unsafe{CloseChannel(ch_id);}
}

// Set input sample rate
pub fn wdsp_set_input_rate(ch_id: i32, in_rate: i32) {
	unsafe{SetInputSamplerate(ch_id, in_rate);}
}

// Set DSP sample rate
pub fn wdsp_set_dsp_rate(ch_id: i32, dsp_rate: i32) {
	unsafe{SetDSPSamplerate(ch_id, dsp_rate);}
}

// Data exchange
pub fn wdsp_exchange(ch_id: i32, in_buf: &mut [f64; (common_defs::DSP_BLK_SZ * 2) as usize],  out_buf: &mut [f64; (common_defs::DSP_BLK_SZ * 2) as usize]) -> i32{	
	let mut error: i32 = 0;
	unsafe{fexchange0(ch_id,  in_buf.as_mut_ptr(),  out_buf.as_mut_ptr(), &mut error as *mut i32)}
	//println!("{}", error);
	return error;	
}

// Modes and filters
pub fn wdsp_set_rx_mode(ch_id: i32, mode: i32) {
	globals::set_mode(mode as u32);
	set_mode_filter(ch_id);
}

pub fn wdsp_set_rx_filter(ch_id: i32, filter: i32) {
	// Filters are 0-7 in order
	// 6K 4K 2.7K 2.4K 2.K1, 1.0K 500Hz 250Hz 100Hz
	globals::set_filter(filter as u32);
	set_mode_filter(ch_id);
}

fn set_mode_filter(ch_id: i32) {
	let mut low: i32 = 0;
	let mut high: i32 = 0;
	
	let filter = globals::get_filter() as i32;
	let mode = globals::get_mode() as i32;
	let new_low;
	let new_high;
	match filter {
		0 => {low = 100; high = 6100},
		1 => {low = 100; high = 4100},
		2 => {low = 300; high = 3000},
		3 => {low = 300; high = 2700},
		4 => {low = 300; high = 2400},
		5 => {low = 300; high = 1300},
		6 => {low = 500; high = 1000},
		7 => {low = 600; high = 850},
		8 => {low = 700; high = 800},
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

	unsafe {
		SetRXAMode(ch_id, mode);
		SetRXABandpassRun(ch_id, 1);
		SetRXABandpassFreqs(ch_id, new_low as f64, new_high as f64);
	}
	
}

// Open WDSP display
pub fn wdsp_open_disp(
	disp_id: i32, fft_size: i32, win_type: i32, 
	sub_spans: i32, in_sz: i32, display_width: i32, 
	average_mode: i32, over_frames: i32, 
	sample_rate: i32, frame_rate: i32) -> bool {

	/*
	** Open a display unit.
	**
	** Arguments:
	**	display			-- display id to use
	** 	fft_size		-- fft size to use, power of 2
	** 	win_type		-- window type
	** 	sub_spans		-- number of receivers to stitch
	** 	in_sz			-- number of input samples
	** 	display_width	-- number of points to plot, generally pixel width
	** 	average_mode	-- modes available :
	** 					-1	Peak detect
	** 					0	No averaging
	** 					1	Time weighted linear
	** 					2	Time weighted log
	** 					3	Window averaging linear
	**					4	Window averaging log
	** 					5	Weighted averaging linear, low noise floor mode
	** 					6	Weighted averaging log, low noise floor mode
	** 	over_frames		-- number of frames to average over
	** 	sample_rate		-- in Hz
	** 	frame_rate		-- required frames per second
	*/
	
	// Create the display analyzer
	let mut success = -1;
	let mut path: i8 = 0;
	unsafe {
		XCreateAnalyzer(
			disp_id,
			&mut success,
			fft_size,
			1,
			sub_spans,
			&mut path
		);
	}
	
	// XCreateAnalyzer sets success to 0 if successful 
	if success == 0 {
		// Calculate the display parameters
		let mut flp: [i32; 1] = [0];
		let overlap: i32 = (f64::max(0.0, f64::ceil(fft_size as f64 - sample_rate as f64 / frame_rate as f64))) as i32;
		let clip_fraction: f64 = 0.17;
		let clp: i32 = f64::floor(clip_fraction * fft_size as f64) as i32;
		//let max_av_frames: i32 = 60;
		let keep_time: f64 = 0.1;
		let max_w: i32 = fft_size + f64::min(keep_time * sample_rate as f64, keep_time * fft_size as f64 * frame_rate as f64) as i32;

		// Set the display parameters
		unsafe {
			SetAnalyzer(
				disp_id,				// the disply id
				1,				// no of LO freq, 1 for non-SA use
				1,					// complex data input
				flp.as_mut_ptr(),	// single value for non-SA use
				fft_size,		// actual fft size same as max fft size for now
				in_sz,			// no input samples per call
				win_type,				// window type
				14.0,				// window shaping function, 14 is recommended
				overlap,			// no of samples to use from previous frame
				clp,					// no of bins to clip off each side of the sub-span
				0,				// no of bins to clip from low end of span (zoom)
				0,				// no of bins to clip from high end of span (zoom)
				display_width,	// no of pixel values to return
				sub_spans,		// no of sub-spans to concatenate to form a complete span
				average_mode,		// select algorithm for averaging
				over_frames,		// number of frames to average over
				0.0,				// not sure how to use this
				0,				// no calibration in use
				0.0,				// min freq for calibration
				0.0,				// max freq for calibration
				max_w,					// how much data to keep in the display buffers
			);
		}
		return true;
	}
	return false;
}

// Update WDSP display parameters
pub fn wdsp_update_disp(
	disp_id: i32, fft_size: i32, win_type: i32, 
	sub_spans: i32, in_sz: i32, display_width: i32, 
	average_mode: i32, over_frames: i32, 
	sample_rate: i32, frame_rate: i32) {

	// Calculate the display parameters
	let mut flp: [i32; 1] = [0];
	let overlap: i32 = (f64::max(0.0, f64::ceil(fft_size as f64 - sample_rate as f64 / frame_rate as f64))) as i32;
	let clip_fraction: f64 = 0.17;
	let clp: i32 = f64::floor(clip_fraction * fft_size as f64) as i32;
	let keep_time: f64 = 0.1;
	let max_w: i32 = fft_size + f64::min(keep_time * sample_rate as f64, keep_time * fft_size as f64 * frame_rate as f64) as i32;

	// Set the display parameters
	unsafe {
		SetAnalyzer(
			disp_id,				// the disply id
			1,				// no of LO freq, 1 for non-SA use
			1,					// complex data input
			flp.as_mut_ptr(),	// single value for non-SA use
			fft_size,		// actual fft size same as max fft size for now
			in_sz,			// no input samples per call
			win_type,				// window type
			14.0,				// window shaping function, 14 is recommended
			overlap,			// no of samples to use from previous frame
			clp,					// no of bins to clip off each side of the sub-span
			0,				// no of bins to clip from low end of span (zoom)
			0,				// no of bins to clip from high end of span (zoom)
			display_width,	// no of pixel values to return
			sub_spans,		// no of sub-spans to concatenate to form a complete span
			average_mode,		// select algorithm for averaging
			over_frames,		// number of frames to average over
			0.0,				// not sure how to use this
			0,				// no calibration in use
			0.0,				// min freq for calibration
			0.0,				// max freq for calibration
			max_w,					// how much data to keep in the display buffers
		);
	}
}

pub fn destroy_analyzer(disp_id: i32) {
	unsafe{ DestroyAnalyzer(disp_id)};
}

// Push display data as interleaved IQ
pub fn wdsp_write_spec_data(disp_id: i32, in_iq: &mut [f32; (common_defs::DSP_BLK_SZ * 2) as usize]) {
	unsafe{ Spectrum2(disp_id, 0, 0, in_iq.as_mut_ptr())};
}

// Get display pixels if available.
pub fn wdsp_get_display_data(disp_id: i32, out_real: &mut [f32; (common_defs::DSP_BLK_SZ) as usize]) -> bool {
	let mut flag: i32 = 0;
	unsafe {GetPixels(disp_id, out_real.as_mut_ptr(), &mut flag);}
	
	if flag == 1 {return true;}
	return false;
}