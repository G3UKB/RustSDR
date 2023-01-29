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
use crate::app::common::globals;
use crate::app::udp::hw_control;

use egui::{RichText, TextStyle};
use eframe::egui;
//use serde:: {Serialize, Deserialize};

#[derive(PartialEq)]
enum Radios { RX1, RX2, RX3 }

//===========================================================================================
// State for Control
pub struct UIControl {
    hw: Rc<RefCell<hw_control::HWData>>,
    prefs: Rc<RefCell<prefs::Prefs>>,
    num_radios: Radios,
    running: bool,
    gain: f32,
}

//===========================================================================================
// Implementation for UIApp
impl UIControl {
    pub fn new(prefs: Rc<RefCell<prefs::Prefs>>, hw: Rc<RefCell<hw_control::HWData>>) -> Self{
        
        let af_gain = prefs.borrow().radio.af_gain;
        Self {
            hw: hw,
            prefs: prefs,
            num_radios: Radios::RX1,
            running: false,
            gain: af_gain,
        }
    }

    //===========================================================================================
    // Populate control window
    pub fn control(&mut self, ui: &mut egui::Ui) {
        
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {

            // Set start button color
            let mut bcolor = egui::Color32::RED;
            if self.running {
                bcolor = egui::Color32::GREEN;
            }

            // Start button
            let b = ui.button(RichText::new("Start")
            .text_style(TextStyle::Monospace)
            .size(16.0)
            .background_color(egui::Color32::TRANSPARENT)
            .color(bcolor));
            if b.clicked() {
                self.hw.borrow_mut().do_start(false);
                self.running = true;
                globals::set_run_state(true);
            }

            // Stop button
            let b = ui.button(RichText::new("Stop")
            .text_style(TextStyle::Monospace)
            .size(16.0)
            .background_color(egui::Color32::TRANSPARENT));
            if b.clicked() {
                if self.running {
                    self.hw.borrow_mut().do_stop();
                    self.running = false;
                    globals::set_run_state(false);
                }
            }

            // Audio gain
            ui.add(egui::Slider::new(&mut self.gain, 0.0..=100.0).suffix("%"));
            self.prefs.borrow_mut().radio.af_gain = self.gain;
            globals::set_af_gain(self.gain);

            // Num RX
            ui.horizontal_wrapped(|ui| {
                if ui.add(egui::RadioButton::new(self.num_radios == Radios::RX1, "RX1")).clicked() {
                    self.num_radios = Radios::RX1
                }
                if ui.add(egui::RadioButton::new(self.num_radios == Radios::RX2, "RX2")).clicked() {
                    self.num_radios = Radios::RX2
                }
                if ui.add(egui::RadioButton::new(self.num_radios == Radios::RX3, "RX3")).clicked() {
                    self.num_radios = Radios::RX3
                }
            });

        });
    }
}