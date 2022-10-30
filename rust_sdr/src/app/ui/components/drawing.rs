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
        draw_line, draw_point, draw_rect_fill, set_draw_color, set_line_style, LineStyle, Offscreen,
    },
    enums::{Color, Event, FrameType},
    frame::Frame,
    prelude::*,
};
use std::cell::RefCell;
use std::rc::Rc;

use crate::app::common::messages;

//==================================================================================
// Drawing State
pub struct DrawingState{
    pub frame : Frame,
}

const WIDTH: i32 = 400;
const HEIGHT: i32 = 200;

// Implementation methods on VFOState
impl DrawingState {
	// Create a new instance and initialise the default arrays
    pub fn new() -> DrawingState {

        // Somewhere to create the widgets
        let mut frame = Frame::default();
        frame.set_size(400, 200);
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
            move |_| {
                let mut offs = offs.borrow_mut();
                if offs.is_valid() {
                    offs.rescale();
                    offs.copy(5, 5, WIDTH - 10, HEIGHT - 10, 0, 0);
                } else {
                    offs.begin();
                    draw_rect_fill(0, 0, WIDTH - 10, HEIGHT - 10, Color::Black);
                    offs.copy(5, 5, WIDTH - 10, HEIGHT - 10, 0, 0);
                    offs.end();
                }
            }
        });

        self.frame.handle({
            let mut x = 0;
            let mut y = 0;
            move |f, ev| {
                // println!("{}", ev);
                // println!("coords {:?}", app::event_coords());
                // println!("get mouse {:?}", app::get_mouse());
                let offs = offs.borrow_mut();
                match ev {
                    Event::Push => {
                        offs.begin();
                        set_draw_color(Color::Red);
                        set_line_style(LineStyle::Solid, 3);
                        let coords = app::event_coords();
                        x = coords.0;
                        y = coords.1;
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
                        draw_line(x, y, coords.0, coords.1);
                        x = coords.0;
                        y = coords.1;
                        offs.end();
                        f.redraw();
                        set_line_style(LineStyle::Solid, 0);
                        true
                    }
                    _ => false,
                }
            }
        });
    }
}