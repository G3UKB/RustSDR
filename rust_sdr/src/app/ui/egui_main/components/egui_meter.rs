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

use crate::app::protocol;
use crate::app::common::globals;
use crate::app::common::common_defs;
use crate::app::ui::egui_main::components;
use crate::app::dsp;

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
            //let desired_size = egui::vec2(100.0, 50.0);
            //let (_id, rect) = ui.allocate_space(desired_size);

            // Get the painter
            let painter = ui.painter();
        });
    }
}