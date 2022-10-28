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
use fltk_grid::Grid;

use crate::app::common::messages;
use crate::app::protocol;
use crate::app::ui::components::main_vfo;

//==================================================================================
// UI State
pub struct UIState{
    app : fltk_app::App,
    i_cc : Arc<Mutex<protocol::cc_out::CCDataMutex>>,
    vfo : main_vfo::VFOState,
    pub ch_r : fltk_app::Receiver<messages::UIMsg>,
}


// Implementation methods on UDPRData
impl UIState {
	// Create a new instance and initialise the default arrays
    pub fn new(i_cc : Arc<Mutex<protocol::cc_out::CCDataMutex>>) -> UIState {
        
        // The one and only fltk app
        let fltk_app = fltk_app::App::default();

        // Create a message channel
        let (s, r) = fltk_app::channel::<messages::UIMsg>();

        // We create all components in-line
        // The main window
        let mut wind = Window::new(100, 100, 400, 300, "RustSDR");

        // The main window is split into areas using a grid layout
        let mut grid = Grid::default_fill();
        grid.set_layout(2, 1);

        // Create the VFO
        let mut vfo = main_vfo::VFOState::new(i_cc.clone(),  s);
        // Initialise and set initial freq
        vfo.init_vfo();
        vfo.set_freq(7300000);
        // Put the VFO in the top grid section
        grid.insert(&mut vfo.frame, 0, 0);
        
        // Assembly end
        wind.end();
        wind.show();
    
        // Object state
        UIState {
            app : fltk_app,
            i_cc : i_cc,
            vfo : vfo,
            ch_r : r,
        }

    }

    // Run the UI event loop
    pub fn run_event_loop(&mut self) {
        //fltk_app::run().unwrap();
        while self.app.wait() {
            // Pick up any of our messages
            if let Some(val) = self.ch_r.recv() {
                match val {
                    messages::UIMsg::FreqUpdate(inc_or_dec) => {
                        self.vfo.inc_dec_freq(inc_or_dec);
                    }
                    _ => println!("No match"),
                }
            }
        }
    }
}