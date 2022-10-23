/*
vfo.rs

Module - vfo
User interface VFO control

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
use std::collections::HashMap;

use fltk::app as fltk_app;
use fltk::{prelude::*, window::Window, frame::Frame};
use fltk::enums::Font;
use fltk::enums::Color;
use fltk_grid::Grid;

use crate::app::protocol;


//==================================================================================
// VFO State
pub struct VFOState{
    i_cc : Arc<Mutex<protocol::cc_out::CCDataMutex>>,
    freq_inc_map : HashMap<i32, i32>,
    digit_map : HashMap<i32, VFODigit>,
    pub frame : Frame,
    pub grid : Grid,
}


// Implementation methods on UDPRData
impl VFOState {
	// Create a new instance and initialise the default arrays
    pub fn new(i_cc : Arc<Mutex<protocol::cc_out::CCDataMutex>>) -> VFOState {

        // Lookup for digit number to frequency increment 100MHz to 1Hz
        let freq_inc_map = HashMap::from([
            (1, 100000000),
            (2, 10000000),
            (3, 1000000),
            (4, 100000),
            (5, 10000),
            (6, 1000),
            (7, 100),
            (8, 10),
            (9, 1),
        ]);

        let digit_map = HashMap::new();

        // Somewhere to create the widgets
        let mut frame = Frame::default();
        let mut grid = Grid::default_fill();

        // Object state
        VFOState {
            i_cc : i_cc,
            freq_inc_map : freq_inc_map,
            digit_map : digit_map,
            frame : frame,
            grid : grid,
        }
    }

    //=========================================================================================
    // Create the set of 9 digits
    pub fn init_vfo(&mut self) {

        // Initialise the grid
        // Accomodate 9 digits and 2 separators
        self.grid.set_layout(1, 11);
        // Create our set of digits
        self.create_digits();
    }

    // Initial freq setting
    pub fn set_freq(&mut self, freq: u32) {
        let new_freq : String = freq.to_string();
        // Need to make this a 9 digit string with leading zeros
        let num_zeros = 9 - new_freq.len();
        let mut zeros_str = String::from("");

        for _i in 0..num_zeros {
            zeros_str += "0";
        }
        let mut freq_str = String::from(zeros_str + &new_freq);


    }

    //=========================================================================================
    // Create the set of 9 digits
    fn create_digits(&mut self) {
        self.grid.insert(&mut Self::new_digit(), 0, 0);
        self.grid.insert(&mut Self::new_digit(), 0, 1);
        self.grid.insert(&mut Self::new_digit(), 0, 2);
        self.grid.insert(&mut Self::new_sep(), 0, 3);
        self.grid.insert(&mut Self::new_digit(), 0, 4);
        self.grid.insert(&mut Self::new_digit(), 0,5);
        self.grid.insert(&mut Self::new_digit(), 0, 6);
        self.grid.insert(&mut Self::new_sep(), 0, 7);
        self.grid.insert(&mut Self::new_digit(), 0, 8);
        self.grid.insert(&mut Self::new_digit(), 0, 9);
        self.grid.insert(&mut Self::new_digit(), 0, 10);
        
    }

    // Create a new separator 
    fn new_sep() -> Frame {
        let mut frame = Frame::default().with_label("_");
        frame.set_label_color(Color::DarkBlue);
        frame.set_label_font(Font::CourierBold);
        frame.set_label_size(20);
        return frame;
    }

    // Set display frequency
    fn set_display_freq(freq : &String) {

    }

}

//==================================================================================
// VFO Digit
pub struct VFODigit{
    id : i32,
    pub frame : Frame,
    i_cc : Arc<Mutex<protocol::cc_out::CCDataMutex>>,
}

// Implementation methods on UDPRData
impl VFODigit {
	// Create a new instance and initialise the default arrays
    pub fn new( id : i32, label : &String, font : Font, size : i32, color : Color, i_cc : Arc<Mutex<protocol::cc_out::CCDataMutex>>) -> VFODigit {

        let mut frame = Frame::default().with_label(label);
        frame.set_label_color(color);
        frame.set_label_font(font);
        frame.set_label_size(size);

        // Object state
        VFODigit {
            id : id,
            frame : frame,
            i_cc : i_cc,
        }
    }

    pub fn get_id(&self) -> i32 {
        return self.id;
    }

    pub fn set_label(&mut self, label : &String) {
        self.frame.set_label(label);
    }
}
