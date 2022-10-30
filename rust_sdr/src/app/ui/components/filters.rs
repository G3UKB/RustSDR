/*
filters.rs

Module - modes
User interface filters panel

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

use std::collections::HashMap;

use fltk::app as fltk_app;
use fltk::{prelude::*, frame::Frame, button::Button};
use fltk::enums::{Font, Color, Event};
use fltk_grid::{Grid, GridRange};

use crate::app::common::messages;
use crate::app::dsp;

//==================================================================================
// Filters State

pub struct FiltersState<'a>{
    pub grid : Grid,
    filter_names : [&'a str; 8],
    button_map : HashMap<i32, Button>,
}

// Implementation methods on VFOState
impl FiltersState<'_> {
	// Create a new instance and initialise the default arrays
    pub fn new() -> FiltersState<'static> {

        // Button names
        let mut filter_names = [
            "6.0KHz",
            "4.0KHz",
            "2.7KHz",
            "2.4KHz",
            "1.0KHz",
            "500Hz",
            "250Hz",
            "100Hz",
        ];

        // Holds all button refs
        let mut button_map = HashMap::new();

        // Somewhere to create the widgets
        let mut grid = Grid::default_fill();

        // Object state
        FiltersState {
            //frame : frame,
            grid : grid,
            filter_names : filter_names,
            button_map : button_map,
        }
    }

    //=========================================================================================
    // Initialise and create widgets
    pub fn init_filters(&mut self) {

        // Initialise the grid
        // Accomodate 8 buttons
        self.grid.set_layout(2, 4);
        // Create our set of buttons
        self.create_filters();
    }

    //=========================================================================================
    // Create the set of 9 digits in 3 sets with separators
    fn create_filters(&mut self) {

        let mut col= 0;
        let mut row= 0;
        for i in 0..8 {
            // Add the next button
            let mut b = self.create_button(
                    i, 
                    &String::from(self.filter_names[i as usize]), 
                    Font::Times, 
                    14, 
                    Color::DarkCyan);
            row = (i/4) as usize;
            col = (i%4) as usize;
            self.grid.insert(&mut b, row, col);
            self.button_map.insert(i as i32, b);
        }
    }

    // Create a new button
    fn create_button(&mut self,
            id : i32, 
            label : &String, 
            font : Font, 
            size : i32, 
            color : Color) -> Button {
        let mut button = Button::default();
        button.set_label(label);
        button.set_color(color);
        button.set_label_font(font);
        button.set_label_size(size);
        button.set_callback ({
            let filter: i32 = id;
            move |b| dsp::dsp_interface::wdsp_set_rx_filter(0, filter)
        });
        return button;
    }
}
