/*
converters.rs
module converters

Convertion of buffer types

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

//
// These are targetted rather than generic conversions. Grouped here for convienience and consistency.
//

// Convert input buffer in i8 BE to output buffer f64 LE
// Input side of DSP
pub fn i8be_to_f64le(in_data: &Vec<u8>, out_data: &mut [f64; (common_defs::DSP_BLK_SZ * 2) as usize]) { 
    // The in_data is a Vec<i8> 1024 complex samples where each each interleaved I and Q are 24 bits in BE format.
    // Thus the length of the input data is 1024*6 representing the 1024 complex samples.
    // The output data is 1024 complex samples in f64 LE format suitable for the DSP exchange function.

    // Scale factors
    let base: i32 = 2;
    let scale: f64 = 1.0 /(base.pow(23)) as f64;
    
    // Size to iterate over
    let sz: u32 = (common_defs::DSP_BLK_SZ * common_defs::BYTES_PER_SAMPLE) - common_defs::BYTES_PER_SAMPLE/2;

    let mut in_index: u32 = 0;
    let mut out_index: u32 = 0;
    let mut as_int: i32;
    
    // Here we would iterate over each receiver and use a 2d array but for now one receiver.
    // Pack the 3 x i8 BE bytes (24 bit sample) into an int in LE format.
    // We must retain the sign hence we shift up to MSB and then down to propogate the sign.
    while in_index <= sz{
        
        // Big endian stores the most significant byte in the lowest address
        // Little endian stores the most significant byte in the highest address
        as_int = 
            ( 
                ((in_data[(in_index+2) as usize] as i32) << 8) | 
                ((in_data[(in_index+1) as usize] as i32) << 16) | 
                ((in_data[(in_index) as usize] as i32) << 24)
            ) >>8;
        // Scale and write to target array
        out_data[out_index as usize] = (as_int as f64) * scale;

        // BYTES_PER_SAMPLE is complex sample but we are moving I and then Q so /2
        in_index += common_defs::BYTES_PER_SAMPLE/2;
        out_index += 1;
    }
}

// Convert input buffer in f64 LE to output buffer i8 BE
// Output side of DSP to hardware
pub fn f64le_to_i8be(sample_sz: usize, in_data: &[f64; (common_defs::DSP_BLK_SZ * 2) as usize], out_data: &mut [u8; common_defs::DSP_BLK_SZ as usize * 8]) {
    // This conversion is the opposite of the i8be_to_f64le() and is the output side of the DSP.
    // The converted data is suitable for insertion into the ring buffer to the UDP writer.

    let base: i32 = 2;
    let scale: f64 = base.pow(15) as f64;

    let mut dest: usize = 0;
    let mut src: usize = 0;
    let mut l: i16;
    let mut r: i16;
    let mut i: i16;
    let mut q: i16;
    
    // We get 1024 f64 audio interleaved left/right
    // We 'will' get f64 samples interleaved IQ output data when TX is implemented
    // This means we have 1024*sizeof f64(8)*left/right(2) bytes of data to iterate on the input
    // However the output is 16 bit packed so we have 1024*2*2 to iterate on the output
    // Both in and out are interleaved

    // We iterate on the output side starting at the LSB
    while dest <= (sample_sz - 8) as usize {
        l = (in_data[src] * scale) as i16;
        r = (in_data[src+1] * scale) as i16;
        i = 0 as i16;
        q = 0 as i16;
        out_data[dest] = ((l >> 8) & 0xff) as u8;
        out_data[dest+1] = (l & 0xff) as u8;
        out_data[dest+2] = ((r >> 8) & 0xff) as u8;
        out_data[dest+3] = (r & 0xff) as u8;

        out_data[dest+4] = i as u8;
        out_data[dest+5] = i as u8;
        out_data[dest+6] = q as u8;
        out_data[dest+7] = q as u8;

        dest += 8;
        src += 2;
    }
}

// Convert input buffer in f64 LE to output buffer i8 LE as f32 values
// Output side of DSP to local audio
pub fn f64le_to_i8le(sample_sz: usize, in_data: &[f64; (common_defs::DSP_BLK_SZ * 2) as usize], out_data: &mut [u8; common_defs::DSP_BLK_SZ as usize * 4]) {
     /*
    * The output data is structured as follows:
    * <L0><L1><L2><L3><R0><R1><R2><R3>...
    *
    * The input is f64 for L and R thus the input size is sizeof f64*2
    * The L and R samples are in f32 format LE. Thus the output sz is sizeof f32*2
    */

    // Copy and encode the samples
    let mut dest: usize = 0;
    let mut src: usize = 0;
    let mut l: i16;
    let mut r: i16;
    let base: i32 = 2;
    let scale: f64 = base.pow(15) as f64;

    // We iterate on the output side starting at the MSB
    while dest <= sample_sz - 4 {
        l = (in_data[src] * scale) as i16;
        r = (in_data[src+1] * scale) as i16;
        
        out_data[dest] = (l & 0xff) as u8;
        out_data[dest+1] = ((l >> 8) & 0xff) as u8;
        out_data[dest+2] = (r & 0xff) as u8;
        out_data[dest+3] = ((r >> 8) & 0xff) as u8;

        dest += 4;
        src += 2;
    }
}

// Convert input buffer in i8 LE to output buffer f32 LE
// Audio ring buffer to local audio
#[allow(unused_parens)]
pub fn i8le_to_f32le(in_data: &Vec<u8>, out_data: &mut Vec<f32>, sz: u32) {
    // The U8 data in the ring buffer is ordered as LE i16 2 byte values 

    let base: i32 = 2;
    let scale: f32 = 1.0 /(base.pow(23)) as f32;
    let mut src: u32 = 0;
    let mut dest: u32 = 0;
    let mut as_int_left: i16;
    let mut as_int_right: i16;
    // NOTE Do not remove parenthesis, they are required
    while src <= sz -4 {
        as_int_left = (
            in_data[src as usize] as i16 | 
            ((in_data[(src+1) as usize] as i16) << 8)); 
        as_int_right = (
            in_data[(src+2) as usize] as i16 | 
            ((in_data[(src+3) as usize] as i16) << 8)); 
        
        // Scale and write to target array
        out_data[dest as usize] = (as_int_left as f32) * scale;
        out_data[(dest+1) as usize] = (as_int_right as f32 * scale);

        src += 4;
        dest += 2;
    }
}