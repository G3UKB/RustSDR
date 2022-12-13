/*
egui_filter.rs

Module - egui_filter
Filter sub-window

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
use std::{cell::RefCell, rc::Rc};

use crate ::app::common::prefs;
use crate::app::protocol;
use crate::app::dsp;
use crate::app::ui::egui_main::components;

use egui::{RichText, TextStyle};
use eframe::egui;
use serde:: {Serialize, Deserialize};

// Filter enumerations
#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone)]
pub enum FilterId {
    F6_0KHz,
    F4_0KHz,
    F2_7KHz,
    F2_4KHz,
    F2_1KHz,
    F1_0KHz,
    F500Hz,
    F250Hz,
    F100Hz,
}

const FILT_NORMAL_COLOR: egui::Color32 = egui::Color32::TRANSPARENT;
const FILT_HIGHLIGHT_COLOR: egui::Color32 = egui::Color32::DARK_RED;

//===========================================================================================
// State for Filters
pub struct UIFilter {
    _i_cc : Arc<Mutex<protocol::cc_out::CCData>>,
    filter: FilterId,
    _filter_width: i32,
    fi_array: [(String, egui::Color32); 9],
    spec : Rc<RefCell<components::egui_spec::UISpec>>,
    prefs: Rc<RefCell<prefs::Prefs>>,
}

//===========================================================================================
// Implementation for UIApp
impl UIFilter {
    pub fn new(_cc: &eframe::CreationContext<'_>, 
        i_cc : Arc<Mutex<protocol::cc_out::CCData>>, 
        spec : Rc<RefCell<components::egui_spec::UISpec>>,
        prefs: Rc<RefCell<prefs::Prefs>>) -> Self{

        let fi_array = [
           (String::from("6K0 "), egui::Color32::TRANSPARENT),
           (String::from("4K0 "), egui::Color32::TRANSPARENT),
           (String::from("2K7 "), egui::Color32::TRANSPARENT),
           (String::from("2K4 "), FILT_HIGHLIGHT_COLOR),
           (String::from("2K1 "), egui::Color32::TRANSPARENT),
           (String::from("1K0 "), egui::Color32::TRANSPARENT),
           (String::from("500H"), egui::Color32::TRANSPARENT),
           (String::from("250H"), egui::Color32::TRANSPARENT),
           (String::from("100H"), egui::Color32::TRANSPARENT),
        ];

        // Retrieve and set filter
        let filter = prefs.borrow().radio.filter;
        dsp::dsp_interface::wdsp_set_rx_mode(0, filter as i32);

        Self {
            _i_cc: i_cc,
            fi_array: fi_array,
            filter: filter,
            _filter_width: 2400,
            prefs: prefs,
            spec: spec,
        }
    }

    //===========================================================================================
    // Populate filters window
    pub fn filters(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            let b = ui.button(RichText::new(&self.fi_array[FilterId::F6_0KHz as usize].0)
            .text_style(TextStyle::Monospace)
            .background_color(self.fi_array[FilterId::F6_0KHz as usize].1));
            if b.clicked() {
                self.spec.borrow_mut().set_filt_width(6000);
                self.filter = FilterId::F6_0KHz;
            }

            let b = ui.button(RichText::new(&self.fi_array[FilterId::F4_0KHz as usize].0)
            .text_style(TextStyle::Monospace)
            .background_color(self.fi_array[FilterId::F4_0KHz as usize].1));
            if b.clicked() {
                self.spec.borrow_mut().set_filt_width(4000);
                self.filter = FilterId::F4_0KHz;
            }

            let b = ui.button(RichText::new(&self.fi_array[FilterId::F2_7KHz as usize].0)
            .text_style(TextStyle::Monospace)
            .background_color(self.fi_array[FilterId::F2_7KHz as usize].1));
            if b.clicked() {
                self.spec.borrow_mut().set_filt_width(2700);
                self.filter = FilterId::F2_7KHz;
            }

            let b = ui.button(RichText::new(&self.fi_array[FilterId::F2_4KHz as usize].0)
            .text_style(TextStyle::Monospace)
            .background_color(self.fi_array[FilterId::F2_4KHz as usize].1));
            if b.clicked() {
                self.spec.borrow_mut().set_filt_width(2400);
                self.filter = FilterId::F2_4KHz;
            }

            let b = ui.button(RichText::new(&self.fi_array[FilterId::F2_1KHz as usize].0)
            .text_style(TextStyle::Monospace)
            .background_color(self.fi_array[FilterId::F2_1KHz as usize].1));
            if b.clicked() {
                self.spec.borrow_mut().set_filt_width(2100);
                self.filter = FilterId::F2_1KHz;
            }

            let b = ui.button(RichText::new(&self.fi_array[FilterId::F1_0KHz as usize].0)
            .text_style(TextStyle::Monospace)
            .background_color(self.fi_array[FilterId::F1_0KHz as usize].1));
            if b.clicked() {
                self.spec.borrow_mut().set_filt_width(1000);
                self.filter = FilterId::F1_0KHz;
            }

            let b = ui.button(RichText::new(&self.fi_array[FilterId::F500Hz as usize].0)
            .text_style(TextStyle::Monospace)
            .background_color(self.fi_array[FilterId::F500Hz as usize].1));
            if b.clicked() {
                self.spec.borrow_mut().set_filt_width(500);
                self.filter = FilterId::F500Hz;
            }

            let b = ui.button(RichText::new(&self.fi_array[FilterId::F250Hz as usize].0)
            .text_style(TextStyle::Monospace)
            .background_color(self.fi_array[FilterId::F250Hz as usize].1));
            if b.clicked() {
                self.spec.borrow_mut().set_filt_width(250);
                self.filter = FilterId::F250Hz;
            }

            let b = ui.button(RichText::new(&self.fi_array[FilterId::F100Hz as usize].0)
            .text_style(TextStyle::Monospace)
            .background_color(self.fi_array[FilterId::F100Hz as usize].1));
            if b.clicked() {
                self.spec.borrow_mut().set_filt_width(100);
                self.filter = FilterId::F100Hz;
            }
            self.prefs.borrow_mut().radio.filter = self.filter;
        });
        self.set_filter_buttons(self.filter as i32);
        dsp::dsp_interface::wdsp_set_rx_filter(0, self.filter as i32);
    }

    // Highlight the selected button
    fn set_filter_buttons(&mut self, id: i32) {
        for i in 0..8 {
            self.fi_array[i as usize].1 = FILT_NORMAL_COLOR;
        }
        self.fi_array[id as usize].1 = FILT_HIGHLIGHT_COLOR;
    }
}