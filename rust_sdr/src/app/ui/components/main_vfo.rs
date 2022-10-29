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

use std::ops::Neg;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::collections::HashMap;

use fltk::app as fltk_app;
use fltk::{prelude::*, frame::Frame};
use fltk::enums::{Font, Color, Event};
use fltk::app::MouseWheel;
use fltk_grid::Grid;

use crate::app::common::messages;
use crate::app::protocol;

//==================================================================================
// VFO State
pub struct VFOState{
    i_cc : Arc<Mutex<protocol::cc_out::CCDataMutex>>,
    freq_inc_map : HashMap<u32, u32>,
    current_freq_in_hz : u32,
    digit_map : HashMap<i32, Frame>,
    //pub frame : Frame,
    pub grid : Grid,
    pub ch_s : fltk_app::Sender<messages::UIMsg>,
}

// Implementation methods on VFOState
impl VFOState {
	// Create a new instance and initialise the default arrays
    pub fn new(i_cc : Arc<Mutex<protocol::cc_out::CCDataMutex>>, ch_s : fltk_app::Sender<messages::UIMsg>) -> VFOState {

        // Lookup for digit number to frequency increment 100MHz to 1Hz
        let freq_inc_map = HashMap::from([
            (0, 100000000),
            (1, 10000000),
            (2, 1000000),
            (3, 100000),
            (4, 10000),
            (5, 1000),
            (6, 100),
            (7, 10),
            (8, 1),
        ]);

        // Hold refs to all digits
        let mut digit_map = HashMap::new();

        // Somewhere to create the widgets
        //let mut frame = Frame::default();
        let mut grid = Grid::default_fill();
        
        // Object state
        VFOState {
            i_cc : i_cc,
            freq_inc_map : freq_inc_map,
            current_freq_in_hz : 0,
            digit_map : digit_map,
            //frame : frame,
            grid : grid,
            ch_s : ch_s,
        }
    }

    //=========================================================================================
    // Initialise and create widgets
    pub fn init_vfo(&mut self) {

        // Initialise the grid
        // Accomodate 9 digits and 2 separators
        self.grid.set_layout(1, 11);
        // Create our set of digits
        self.create_digits();
    }

    // Set frequency
    pub fn set_freq(&mut self, freq: u32) {
        self.current_freq_in_hz = freq;
        let new_freq : String = freq.to_string();
        // Need to make this a 9 digit string with leading zeros
        let num_zeros = 9 - new_freq.len();
        let mut zeros_str = String::from("");

        for _i in 0..num_zeros {
            zeros_str += "0";
        }
        let mut freq_str = String::from(zeros_str + &new_freq);
        self.set_display_freq(&freq_str);
    }

    //=========================================================================================
    // Create the set of 9 digits in 3 sets with separators
    fn create_digits(&mut self) {

        let mut index = 0;
        for i in 0..11 {
            if (i == 3) || (i == 7) {
                // Add a separator
                let mut sep = self.new_sep();
                self.grid.insert(&mut sep, 0, i);
            } else {
                // Add the next digit
                let mut digit = self.create_digit(
                        index, 
                        &String::from("0"), 
                        Font::Times, 
                        30, 
                        Color::DarkCyan);
                self.grid.insert(&mut digit, 0, i);
                self.digit_map.insert(index as i32, digit);
                index += 1;
            }
        }
    }

    // Create a new separator 
    fn new_sep(&mut self) -> Frame {
        let mut frame = Frame::default().with_label("_");
        frame.set_label_color(Color::DarkBlue);
        frame.set_label_font(Font::CourierBold);
        frame.set_label_size(20);
        return frame;
    }

    // Create a new digit
    fn create_digit(&mut self,
            id : i32, 
            label : &String, 
            font : Font, 
            size : i32, 
            color : Color) -> Frame {
        let mut frame = Frame::default().with_label(label);
        frame.set_label_color(color);
        frame.set_label_font(font);
        frame.set_label_size(size);
        // Handle mouse events
        frame.handle({
            // Bring variables into closure
            // CC_out instance
            let cc = self.i_cc.clone();
            // id for this digit
            let w_id: i32 = id;
            // freq increment for this digit
            let freq_inc = (self.freq_inc_map[&(w_id as u32)]) as i32;
            let freq_dec = freq_inc.neg();
            let ch_s = self.ch_s;
            move |f, ev| match ev {
                Event::Enter => {
                    // Grow the label when we mouse over
                    f.set_label_size(35);
                    f.redraw_label();
                    true
                }
                Event::Leave => {
                    // Shrink the label back again
                    f.set_label_size(30);
                    f.redraw_label();
                    true
                }
                Event::MouseWheel => {
                    // Here we need to increment/decrement the frequency
                    // This will also reset the display and update the radio
                    let mut inc_or_dec: i32 = 0;
                    match fltk::app::event_dy() {
                        MouseWheel::None => (),
                        MouseWheel::Up => inc_or_dec = freq_dec,
                        MouseWheel::Down => inc_or_dec = freq_inc,
                        MouseWheel::Right => (),
                        MouseWheel::Left => (),
                    }
                    // Message to event loop
                    // Note .emit does not work for some reason
                    //f.emit(ch_s, messages::UIMsg::FreqUpdate(inc_or_dec));
                    ch_s.send(messages::UIMsg::FreqUpdate(inc_or_dec));
                    true
                }
                _ => false
            }
        });
        return frame;
    }

    // Increment or Decrement frequency by the amount of the digit weight
    pub fn inc_dec_freq(&mut self, inc_or_dec: i32) {
        // Update current freq holder
        self.current_freq_in_hz = (self.current_freq_in_hz as i32 + inc_or_dec) as u32;
        // Update the display
        self.set_freq(self.current_freq_in_hz);
        // Update the radio
        self.i_cc.lock().unwrap().cc_set_rx_tx_freq(self.current_freq_in_hz);
    }

    // Set display frequency
    fn set_display_freq(&mut self, freq : &String) {
        for i in 0..freq.len() {
            let mut digit = self.digit_map.get_mut(&(i as i32)).unwrap();
            digit.set_label(&freq.chars().nth(i).unwrap().to_string());
        }

    }

}

