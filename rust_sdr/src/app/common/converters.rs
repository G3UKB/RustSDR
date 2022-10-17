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
pub fn i8be_to_f64le(in_data: &Vec<u8>, out_data: &mut [f64; (common_defs::DSP_BLK_SZ * 2) as usize], scale: f64, sz: u32) { 
    // The in_data is a Vec<i8> 1024 complex samples where each each interleaved I and Q are 24 bits in BE format.
    // Thus the length of the input data is 1024*6 representing the 1024 complex samples.
    // The output data is 1024 complex samples in f64 LE format suitable for the DSP exchange function.

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
                ((in_data[in_index as usize] as i32) << 24)
            ) >>8;

        // Scale and write to target array
        out_data[out_index as usize] = (as_int as f64) * scale;

        // BYTES_PER_SAMPLE is complex sample but we are moving I and then Q so /2
        in_index += common_defs::BYTES_PER_SAMPLE/2;
        out_index += 1;
    }
}

// Convert input buffer in f64 LE to output buffer i8 BE
pub fn f64le_to_i8be(in_data: &[f64; (common_defs::DSP_BLK_SZ * 2) as usize], out_data: &mut [u8; common_defs::DSP_BLK_SZ as usize * 8], scale: f64, sz: u32) {
    // This conversion is the opposite of the i8be_to_f64le() and is output side of the DSP
    // The converted data is suitable for insertion into the ring buffer to the UDP writer.

    //let out_sz: usize = (common_defs::DSP_BLK_SZ * 4 * 2) as usize;
    //let base: i32 = 2;
    //let output_scale: f64 = base.pow(15) as f64;
    let mut dest: usize = 0;
    let mut src: usize = 0;
    let mut L: i16;
    let mut R: i16;
    let mut I: i16;
    let mut Q: i16;
    
    // We iterate on the output side starting at the LSB
    while dest <= (sz - 8) as usize {
        L = (in_data[src] * scale) as i16;
        R = (in_data[src+1] * scale) as i16;
        I = 0 as i16;
        Q = 0 as i16;
        out_data[dest] = ((L >> 8) & 0xff) as u8;
        out_data[dest+1] = (L & 0xff) as u8;
        out_data[dest+2] = ((R >> 8) & 0xff) as u8;
        out_data[dest+3] = (R & 0xff) as u8;

        out_data[dest+4] = I as u8;
        out_data[dest+5] = I as u8;
        out_data[dest+6] = Q as u8;
        out_data[dest+7] = Q as u8;

        dest += 8;
        src += 2;
    }
}

// Convert input buffer in f64 LE to output buffer i8 LE
pub fn f64le_to_i8le() {

}

// Convert input buffer in i8 LE to output buffer f32 LE
pub fn i8le_to_f32le() {

}