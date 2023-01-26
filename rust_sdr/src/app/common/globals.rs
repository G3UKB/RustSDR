/*
globals.rs

Module - globals
Global objects

Copyright (C) 2023 by G3UKB Bob Cowdery

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

use std::sync::atomic::{AtomicU32, Ordering};

use crate::app::common::common_defs;

//========================================================================
// Globals are not a generally good idea but sometimes the best way to solve a problem.
// We require a easy means for the UI to communicate some dynamic settings to the rest
// of the program. These settings might be used in a number of modules. The linkages would 
// be pretty horrendous to manage. This is neat and easy to manage.

static AUDIO_GAIN: AtomicU32 = AtomicU32::new(common_defs::AUDIO_GAIN);

pub fn get_audio_gain() -> u32 {
    AUDIO_GAIN.load(Ordering::Relaxed)
}

pub fn set_audio_gain(gain: u32) {
    AUDIO_GAIN.store(gain, Ordering::Relaxed);
} 
