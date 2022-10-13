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

use cpal::{Data, Sample, SampleFormat};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::app::common::ringb;

//==================================================================================
// Audio output 
pub struct AudioData<'a> {
    rb_audio : &'a ringb::SyncByteRingBuf,
}

impl AudioData<'_> {
	// Create a new instance and initialise the default data
	pub fn new(rb_audio : & ringb::SyncByteRingBuf) -> AudioData {
		
        AudioData {
            rb_audio : rb_audio,
        }
    }

    // Create an audio output stream
    pub fn init_audio() {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("no output device available");

        let mut supported_configs_range = device.supported_output_configs()
        .expect("error while querying configs");
        let supported_config = supported_configs_range.next()
        .expect("no supported config?!")
        .with_max_sample_rate();

        let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);
        let sample_format = supported_config.sample_format();
        let config = supported_config.into();
        let stream = match sample_format {
        SampleFormat::F32 => device.build_output_stream(&config, Self::write_audio::<f32>, err_fn),
        SampleFormat::I16 => device.build_output_stream(&config, Self::write_audio::<i16>, err_fn),
        SampleFormat::U16 => device.build_output_stream(&config, Self::write_audio::<u16>, err_fn),
        }.unwrap();

        // Start the default stream
        stream.play().unwrap();
    }

    // Callback when the audio output needs more data
    fn write_audio<T: Sample>(data: &mut [T], _: &cpal::OutputCallbackInfo) {
        // Check the ring buffer for data
        for sample in data.iter_mut() {
            *sample = Sample::from(&0.0);
        }
    }
}
