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

use lazy_static::lazy_static;
use std::sync::Mutex;
use std::collections::HashMap;

use crate::app::common::common_defs;

//========================================================================
// Globals are not a generally good idea but sometimes the best way to solve a problem.
// We require a easy means for the global state to be set and accessed by any module.

lazy_static! {
    static ref BOOL_SETTINGS: Mutex<HashMap<String, bool>> = Mutex::new(HashMap::new());
    static ref INT_SETTINGS: Mutex<HashMap<String, u32>> = Mutex::new(HashMap::new());
    static ref FLOAT_SETTINGS: Mutex<HashMap<String, f32>> = Mutex::new(HashMap::new());
    static ref STR_SETTINGS: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

//========================================
pub fn get_discover_state() -> bool {
    match BOOL_SETTINGS.lock().unwrap().get("DISCOVER") {
        Some(state) => return state.clone(),
        None => return false,
    }
}

pub fn set_discover_state(state: bool) {
    BOOL_SETTINGS.lock().unwrap().insert("DISCOVER".to_string(), state);
}

//========================================
pub fn get_run_state() -> bool {
    match BOOL_SETTINGS.lock().unwrap().get("RUN_STATE") {
        Some(state) => return state.clone(),
        None => return false,
    }
}

pub fn set_run_state(state: bool) {
    BOOL_SETTINGS.lock().unwrap().insert("RUN_STATE".to_string(), state);
}

//========================================
pub fn get_af_gain() -> f32 {
    match FLOAT_SETTINGS.lock().unwrap().get("AUDIO_GAIN") {
        Some(gain) => return gain.clone(),
        None => return common_defs::AUDIO_GAIN,
    }
}

pub fn set_af_gain(gain: f32) {
    FLOAT_SETTINGS.lock().unwrap().insert("AUDIO_GAIN".to_string(), gain);
}

//========================================
pub fn get_num_rx() -> u32 {
    match INT_SETTINGS.lock().unwrap().get("NUM_RX") {
        Some(num_rx) => return num_rx.clone(),
        None => return common_defs::NUM_RX,
    }
}

pub fn set_num_rx(num_rx: u32) {
    INT_SETTINGS.lock().unwrap().insert("NUM_RX".to_string(), num_rx);
} 

//========================================
pub fn get_smpl_rate() -> u32 {
    match INT_SETTINGS.lock().unwrap().get("SMPL_RATE") {
        Some(rate) => return rate.clone(),
        None => return common_defs::SAMPLE_RATE,
    }
}

pub fn set_smpl_rate(rate: u32) {
    INT_SETTINGS.lock().unwrap().insert("SMPL_RATE".to_string(), rate);
}

//========================================
pub fn get_mode() -> u32 {
    match INT_SETTINGS.lock().unwrap().get("MODE") {
        Some(mode) => return mode.clone(),
        None => return 0,
    }
}

pub fn set_mode(mode: u32) {
    INT_SETTINGS.lock().unwrap().insert("MODE".to_string(), mode);
}

//========================================
pub fn get_filter() -> u32 {
    match INT_SETTINGS.lock().unwrap().get("FILTER") {
        Some(filter) => return filter.clone(),
        None => return 0,
    }
}

pub fn set_filter(filter: u32) {
    INT_SETTINGS.lock().unwrap().insert("FILTER".to_string(), filter);
}