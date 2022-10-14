/*
audio_out.rs

Module - audio_out
manages local audio out

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

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Data, Sample, SampleFormat, PlayStreamError};
use std::vec;
use std::io::Read;
use std::sync::Arc;

use crate::app::common::ringb;

//==================================================================================
// Audio output
pub struct AudioData {
    rb_audio: Arc<ringb::SyncByteRingBuf>,
    //stream: Option<cpal::Stream>,
}

impl AudioData {
    // Create a new instance and initialise the default data
    pub fn new(rb_audio: Arc<ringb::SyncByteRingBuf>) -> AudioData {
        AudioData {
            rb_audio: rb_audio,
           //stream: None,
        }
    }

    // Create an audio output stream
    pub fn init_audio(&mut self) -> cpal::Stream {
        println!("Initialising local audio...");
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("no output device available");

        let mut supported_configs_range = device
            .supported_output_configs()
            .expect("error while querying configs");
        let supported_config = supported_configs_range
            .next()
            .expect("no supported config?!")
            .with_max_sample_rate();

        let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);
        let sample_format = supported_config.sample_format();
        let config = supported_config.into();
        let rb_audio = self.rb_audio.clone();
        let stream = match sample_format {
            SampleFormat::F32 => device.build_output_stream(
                &config,
                move |data, info| write_audio::<f32>(data, info, &rb_audio),
                err_fn,
            ),
            SampleFormat::I16 => device.build_output_stream(
                &config,
                move |data, info| write_audio::<i16>(data, info, &rb_audio),
                err_fn,
            ),
            SampleFormat::U16 => device.build_output_stream(
                &config,
                move |data, info| write_audio::<u16>(data, info, &rb_audio),
                err_fn,
            ),
        }
        .unwrap();

        println!("Starting audio stream");
        stream.play().unwrap();
        //self.stream = Some(stream);
        return stream;

    } 
    
    // Close stream
    pub fn close_audio(&mut self, stream: &cpal::Stream) {
        println!("Closing audio stream");
    }

}

// Callback when the audio output needs more data
fn write_audio<T: Sample>(data: &mut [f32], _: &cpal::OutputCallbackInfo, rb_audio: &ringb::SyncByteRingBuf) {
    // Byte data from ring buffer
    //println!("Len {}", data.len());
    
    let mut rb_data: Vec<u8> = vec![0; data.len()*4];
    let mut in_data: Vec<f32> = vec![0.0; data.len()];
    let mut i = 0;

    //let audio_data = rb_audio.read().read(&mut rb_data);
    //match audio_data {
    //    Ok(_sz) => {
            for sample in data.iter_mut() {
                *sample = in_data[i];
                i += 1;
            }
    //    }
    //    Err(e) => println!("Read error on rb_iq {:?}. Skipping cycle.", e),
    //}
    
}

// Convert a Vec:u8 to a Vec:f32
fn u8_to_f32() {
    // The U8 data in the ring buffer is ordered as LE 16 bit values

}

