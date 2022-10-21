/*
main_window.rs

Module - main_window
User interface main window

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

use std::sync::{Arc, Mutex};

use fltk::app as fltk_app;
use fltk::{prelude::*, window::Window};

use crate::app::protocol;

//==================================================================================
// UI State
pub struct UIState{
    i_cc : Arc<Mutex<protocol::cc_out::CCDataMutex>>,
}


// Implementation methods on UDPRData
impl UIState {
	// Create a new instance and initialise the default arrays
    pub fn new(i_cc : Arc<Mutex<protocol::cc_out::CCDataMutex>>) -> UIState {

        UIState {
            i_cc : i_cc,
        }
    }


    //=========================================================================================
    // Create main application window
    pub fn init_main_window(&mut self) {
        let fltk_app = fltk_app::App::default();
        let mut wind = Window::new(100, 100, 400, 300, "RustSDR");
        wind.end();
        wind.show();
        
    }

    pub fn run_event_loop(&mut self) {
        fltk_app::run().unwrap();
    }

    pub fn set_freq(&mut self, freq: u32) {
        self.i_cc.lock().unwrap().cc_set_rx_tx_freq(freq);
    }

}


