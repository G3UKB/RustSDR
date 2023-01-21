/*
egui_control.rs

Module - egui_control
Mode sub-window

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

use std::{cell::RefCell, rc::Rc};

use crate ::app::common::prefs;
use crate::app::udp::hw_control;

use egui::{RichText, TextStyle};
use eframe::egui;
//use serde:: {Serialize, Deserialize};


//===========================================================================================
// State for Control
pub struct UIControl {
    hw: Rc<RefCell<hw_control::HWData>>,
    _prefs: Rc<RefCell<prefs::Prefs>>,
}

//===========================================================================================
// Implementation for UIApp
impl UIControl {
    pub fn new(prefs: Rc<RefCell<prefs::Prefs>>, hw: Rc<RefCell<hw_control::HWData>>) -> Self{
        
        Self {
            hw: hw,
            _prefs: prefs,
        }
    }

    //===========================================================================================
    // Populate control window
    pub fn control(&mut self, ui: &mut egui::Ui) {
        
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {

            let b = ui.button(RichText::new("Start")
            .text_style(TextStyle::Monospace)
            .size(16.0)
            .background_color(egui::Color32::DARK_GRAY));
            if b.clicked() {
                self.hw.borrow_mut().do_start(false);
            }

            let b = ui.button(RichText::new("Stop")
            .text_style(TextStyle::Monospace)
            .size(16.0)
            .background_color(egui::Color32::DARK_GRAY));
            if b.clicked() {
                self.hw.borrow_mut().do_stop();
            }
        });
    }
}