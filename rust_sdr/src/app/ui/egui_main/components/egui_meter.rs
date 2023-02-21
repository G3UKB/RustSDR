/*
egui_meter.rs

Module - egui_meter
M<etering sub-window

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

use std::sync::{Arc, Mutex};
use std::{cell::RefCell, rc::Rc};

use epaint::Color32;

use crate::app::protocol;
use crate::app::common::globals;
use crate::app::common::common_defs;
use crate::app::ui::egui_main::components;
use crate::app::dsp;

const TEXT_COLOR: Color32 = Color32::from_rgba_premultiplied(150,0,0,70);
const GRID_COLOR: Color32 = Color32::from_rgba_premultiplied(0,50,0,10);
const DYN_COLOR: Color32 = Color32::from_rgba_premultiplied(150,150,0,70);

//===========================================================================================
// State for meter
pub struct UIMeter {
    // Parameters
}

//===========================================================================================
// Implementation for UIMeter
impl UIMeter {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self{

        Self {
        }
    }


    pub fn meter(&mut self, ui: &mut egui::Ui) {
        
        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            // Ensure repaint
            ui.ctx().request_repaint();

            // Go with the maximum available width and keep the aspect ratio constant
            //let desired_size = ui.available_width() * egui::vec2(1.0, 0.5);
            let desired_size = egui::vec2(200.0, 50.0);
            let (_id, rect) = ui.allocate_space(desired_size);

            // Get the painter
            let painter = ui.painter();

            // Draw legends
            let s = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "+20", "+40"];

            for i in 0..s.len() {
                painter.text(
                    egui::pos2(rect.left() + 5.0 + (i as f32*17.0), rect.bottom() - 10.0),
                    egui::Align2::LEFT_CENTER,
                    &String::from(s[i]),
                    egui::FontId::new(10.0,egui::FontFamily::Proportional),
                    TEXT_COLOR,
                );
            }

            painter.line_segment(
                [
                    egui::pos2(rect.left() + 5.0, rect.bottom() - 20.0),
                    egui::pos2(rect.right() - 5.0, rect.bottom() - 20.0),
                ],
            egui::Stroke::new(0.5, GRID_COLOR),
            );
        });
    }
}