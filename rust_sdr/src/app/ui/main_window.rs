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
use fltk::{prelude::*, group::Group, window::Window, frame::Frame};
use fltk_grid::Grid;

use crate::app::common::messages;
use crate::app::protocol;
use crate::app::ui::components::main_vfo;
use crate::app::ui::components::modes;
use crate::app::ui::components::filters;

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


        //========================================================================
        // Assemble the UI
        // The main window
        let mut wind = Window::new(100, 100, 400, 140, "RustSDR");

        // The main window is split into areas using a grid layout
        let mut grid = Grid::default_fill();
        grid.set_layout(2, 2);
        
        // Put the VFO in the top grid section
        let mut vfo_group = Group::new(0,0,200,60, "");
        // Initialise and set initial freq
        let mut vfo = main_vfo::VFOState::new(i_cc.clone(),  s);
        vfo.init_vfo();
        vfo.set_freq(7300000);
        grid.insert(&mut vfo_group, 0, 0..1);
        vfo_group.end();
        
        // Put the modes in the bottom grid left section
        let mut modes_group = Group::new(0,0,200,60, "");
        let mut modes = modes::ModesState::new();
        // Initialise
        modes.init_modes();
        grid.insert(&mut modes_group, 1, 0);
        modes_group.end();

        // Put the filters in the bottom grid right section
        let mut filters_group = Group::new(200,0,200,60, "");
        let mut filters = filters::FiltersState::new();
        // Initialise
        filters.init_filters();
        grid.insert(&mut filters_group, 1, 1);
        filters_group.end();

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