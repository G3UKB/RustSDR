/*
drawing.rs

Module - drawing
User interface drawing control

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

use fltk::{
    app,
    draw::{
        draw_text, draw_line, draw_point, draw_rect_fill, set_draw_color, set_line_style, LineStyle, Offscreen, draw_text2,
    },
    enums::{Color, Event, FrameType, Align},
    frame::Frame,
    prelude::*,
};
use std::cell::RefCell;
use std::rc::Rc;
use core::cell::RefMut;

use crate::app::common::messages;

//==================================================================================
// Drawing State
pub struct DrawingState{
    pub frame : Frame,
}

const WIDTH: i32 = 400;
const HEIGHT: i32 = 200;
const LOW_DB: i32 = -140;
const HIGH_DB: i32 = -20;
const Y_V_LABEL_ADJ: i32 = -3;
const L_MARGIN: i32 = 35;
const R_MARGIN: i32 = 15;
const T_MARGIN: i32 = 10;
const B_MARGIN: i32 = 10;
const TEXT_COLOR: Color = Color::Red;
const GRID_COLOR: Color = Color::Yellow;

// Implementation methods on VFOState
impl DrawingState {
	// Create a new instance and initialise the default arrays
    pub fn new() -> DrawingState {

        // Somewhere to create the widgets
        let mut frame = Frame::default();
        frame.set_size(WIDTH, HEIGHT);
        frame.set_pos(0,120);
        frame.set_color(Color::Black);

        // Object state
        DrawingState {
            frame : frame,
        }
    }

    pub fn init(&mut self) {
        // We fill our offscreen with black
        let offs = Offscreen::new(self.frame.width(), self.frame.height()).unwrap();
        {
            offs.begin();
            draw_rect_fill(0, 0, WIDTH - 10, HEIGHT - 10, Color::Black);
            offs.end();
        }

        let offs = Rc::from(RefCell::from(offs));
        self.frame.draw({
            let offs = offs.clone();
            move |f| {
                DrawingState::draw_static(offs.borrow_mut());
                f.redraw();
            }
        });
/* 
        self.frame.handle({
            let mut x = 0;
            let mut y = 0;
            move |f, ev| {
                 //println!("{}", ev);
                 //println!("coords {:?}", app::event_coords());
                 //println!("get mouse {:?}", app::get_mouse());
                let offs = offs.borrow_mut();
                match ev {
                    Event::Push => {
                        offs.begin();
                        set_draw_color(Color::Red);
                        set_line_style(LineStyle::Solid, 3);
                        let coords = app::event_coords();
                        x = coords.0;
                        y = coords.1 - 125;
                        draw_point(x, y);
                        offs.end();
                        f.redraw();
                        set_line_style(LineStyle::Solid, 0);
                        true
                    }
                    Event::Drag => {
                        offs.begin();
                        set_draw_color(Color::Red);
                        set_line_style(LineStyle::Solid, 3);
                        let coords = app::event_coords();
                        draw_line(x, y, coords.0, coords.1-125);
                        x = coords.0;
                        y = coords.1-125;
                        offs.end();
                        f.redraw();
                        set_line_style(LineStyle::Solid, 0);
                        true
                    }
                    _ => false,
                }
            }
        });
        */
    }

    
    // Static drawing of grid and labels
    fn draw_static(mut offs: RefMut<Offscreen> ) {
        if offs.is_valid() {
            offs.rescale();
            offs.copy(5, 125, WIDTH - 10, HEIGHT - 10, 0, 0);
        } else {
            draw_rect_fill(0, 0, WIDTH - 10, HEIGHT - 10, Color::Black);
            offs.copy(5, 125, WIDTH - 10, HEIGHT - 10, 0, 0);
        }
        offs.begin();
        DrawingState::draw_horizontal();
        offs.end();
        offs.copy(5, 125, WIDTH - 10, HEIGHT - 10, 0, 0);
    }
    
    // Draw horizontal lines at 20 db intervals
    fn draw_horizontal() {
        let db_divs = (LOW_DB.abs() - HIGH_DB.abs()) / 20;
        let db_pixels_per_div: f32 = ((HEIGHT - T_MARGIN - B_MARGIN) as f32 / db_divs as f32);
        let mut j = HIGH_DB;
        for i in 0..db_divs {
            set_draw_color(TEXT_COLOR);
            set_line_style(LineStyle::Solid, 1);
            draw_text2(&String::from(j.to_string()), 5, Y_V_LABEL_ADJ + (i*db_pixels_per_div as i32), 40, 20, Align::Left);
            set_draw_color(GRID_COLOR);
            draw_line(L_MARGIN, T_MARGIN + (i*db_pixels_per_div as i32), WIDTH-R_MARGIN, T_MARGIN + (i*db_pixels_per_div as i32));
            j -= 20;
        }
    }
}