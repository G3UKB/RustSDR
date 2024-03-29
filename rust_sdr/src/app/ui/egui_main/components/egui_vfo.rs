/*
egui_vfo.rs

Module - egui_vfo
VFO sub-window

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
use std::ops::Neg;
use std::{cell::RefCell, rc::Rc};

use crate ::app::common::globals;
use crate ::app::common::prefs;
use crate::app::protocol;

use egui::{RichText, TextStyle};
use eframe::egui;

// VFO enumeration
enum VfoId {
    F100M,
    F10M,
    F1M,
    F100K,
    F10K,
    F1K,
    F100H,
    F10H,
    F1H,
}

const MHZ_SZ: f32 = 55.0;
const KHZ_SZ: f32 = 55.0;
const HZ_SZ: f32 = 35.0;
const VFO_NORMAL_COLOR: egui::Color32 = egui::Color32::TRANSPARENT;
const VFO_HIGHLIGHT_COLOR: egui::Color32 = egui::Color32::DARK_GREEN;

//===========================================================================================
// State for VFO
pub struct UIVfo {
    i_cc : Arc<Mutex<protocol::cc_out::CCData>>,
    f_array: [(String, f32, egui::Color32); 9],
    frequency: u32,
    prefs: Rc<RefCell<prefs::Prefs>>,
}

//===========================================================================================
// Implementation for UIApp
impl UIVfo {
    pub fn new(_cc: &eframe::CreationContext<'_>, i_cc : Arc<Mutex<protocol::cc_out::CCData>>, prefs: Rc<RefCell<prefs::Prefs>>) -> Self{

        let f_array = [
           (String::from("0"), MHZ_SZ, egui::Color32::TRANSPARENT),
           (String::from("0"), MHZ_SZ, egui::Color32::TRANSPARENT),
           (String::from("7"), MHZ_SZ, egui::Color32::TRANSPARENT),
           (String::from("1"), KHZ_SZ, egui::Color32::TRANSPARENT),
           (String::from("0"), KHZ_SZ, egui::Color32::TRANSPARENT),
           (String::from("0"), KHZ_SZ, egui::Color32::TRANSPARENT),
           (String::from("0"), HZ_SZ, egui::Color32::TRANSPARENT),
           (String::from("0"), HZ_SZ, egui::Color32::TRANSPARENT),
           (String::from("0"), HZ_SZ, egui::Color32::TRANSPARENT), 
        ];

        // Which RX are we
        let rx = globals::get_sel_rx();
        // Retrieve and set freq
        let mut freq = prefs.borrow().radio.rx1.frequency;
        match rx {
            1 => {
                freq = prefs.borrow().radio.rx1.frequency;
                i_cc.lock().unwrap().cc_set_rx_tx_freq(freq);
            },
            2 => {
                freq = prefs.borrow().radio.rx2.frequency;
                i_cc.lock().unwrap().cc_set_rx2_freq(freq);
            },
            3 => {
                freq = prefs.borrow().radio.rx3.frequency;
                i_cc.lock().unwrap().cc_set_rx3_freq(freq);
            },
            _ => (),

        }

        Self {
            i_cc: i_cc,
            f_array: f_array,
            frequency: freq,
            prefs: prefs,
        }
    }

    //===========================================================================================
    // Populate VFO window
    pub fn vfo(&mut self, ui: &mut egui::Ui) {

        self.restore_freq();
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.add_space(100.0);
            ui.style_mut().spacing.item_spacing = egui::vec2(14.0,5.0);
            
            let f_100_m = ui.label(RichText::new(&self.f_array[VfoId::F100M as usize].0).text_style(TextStyle::Heading)
            .size(self.f_array[VfoId::F100M as usize].1)
            .background_color(self.f_array[VfoId::F100M as usize].2)
            .strong());
            self.scroll_if(ui, VfoId::F100M, f_100_m.rect, 100000000);

            let f_10_m = ui.label(RichText::new(&self.f_array[VfoId::F10M as usize].0).text_style(TextStyle::Heading)
            .size(self.f_array[VfoId::F10M as usize].1)
            .background_color(self.f_array[VfoId::F10M as usize].2)
            .strong());
            self.scroll_if(ui, VfoId::F10M ,f_10_m.rect, 10000000);

            let f_1_m = ui.label(RichText::new(&self.f_array[VfoId::F1M as usize].0).text_style(TextStyle::Heading)
            .size(self.f_array[VfoId::F1M as usize].1)
            .background_color(self.f_array[VfoId::F1M as usize].2)
            .strong());
            self.scroll_if(ui, VfoId::F1M, f_1_m.rect, 1000000);

            ui.label(RichText::new("-").text_style(TextStyle::Heading).strong()
            .size(30.0));

            let f_100_k = ui.label(RichText::new(&self.f_array[VfoId::F100K as usize].0).text_style(TextStyle::Heading)
            .size(self.f_array[VfoId::F100K as usize].1)
            .background_color(self.f_array[VfoId::F100K as usize].2)
            .strong());
            self.scroll_if(ui, VfoId::F100K, f_100_k.rect, 100000);

            let f_10_k = ui.label(RichText::new(&self.f_array[VfoId::F10K as usize].0).text_style(TextStyle::Heading)
            .size(self.f_array[VfoId::F10K as usize].1)
            .background_color(self.f_array[VfoId::F10K as usize].2)
            .strong());
            self.scroll_if(ui, VfoId::F10K, f_10_k.rect, 10000);

            let f_1_k = ui.label(RichText::new(&self.f_array[VfoId::F1K as usize].0).text_style(TextStyle::Heading)
            .size(self.f_array[VfoId::F1K as usize].1)
            .background_color(self.f_array[VfoId::F1K as usize].2)
            .strong());
            self.scroll_if(ui,VfoId::F1K, f_1_k.rect, 1000);

            ui.label(RichText::new("-").text_style(TextStyle::Heading).strong()
            .size(30.0));

            let f_100_h = ui.label(RichText::new(&self.f_array[VfoId::F100H as usize].0).text_style(TextStyle::Heading)
            .size(self.f_array[VfoId::F100H as usize].1)
            .background_color(self.f_array[VfoId::F100H as usize].2)
            .strong());
            self.scroll_if(ui, VfoId::F100H, f_100_h.rect, 100);

            let f_10_h = ui.label(RichText::new(&self.f_array[VfoId::F10H as usize].0).text_style(TextStyle::Heading)
            .size(self.f_array[VfoId::F10H as usize].1)
            .background_color(self.f_array[VfoId::F10H as usize].2)
            .strong());
            self.scroll_if(ui, VfoId::F10H,f_10_h.rect, 10);

            let f_1_h = ui.label(RichText::new(&self.f_array[VfoId::F1H as usize].0).text_style(TextStyle::Heading)
            .size(self.f_array[VfoId::F1H as usize].1)
            .background_color(self.f_array[VfoId::F1H as usize].2)
            .strong());
            self.scroll_if(ui, VfoId::F1H, f_1_h.rect, 1);
        });
        self.set_freq();
        self.freq_updated();
    }

    // If within the rectangle of a digit then highlight the digit, else normal.
    // If the mouse wheel is being scrolled then scroll the digit up or down.
    fn scroll_if(&mut self, ui: &mut egui::Ui, id: VfoId, r: egui::Rect, inc_or_dec: i32) {
        if ui.rect_contains_pointer(r) {
            //self.f_array[id as usize].1 = grow;
            self.f_array[id as usize].2 = VFO_HIGHLIGHT_COLOR; 
            let e = &ui.ctx().input().events;
            if e.len() > 0 {
                match &e[0] {
                    egui::Event::Scroll(v) => {
                        let mut dir = inc_or_dec;
                        if v[1] < 0.0 {
                            dir = dir.neg();
                        }
                        self.frequency = (self.frequency as i32 + dir) as u32;
                    }
                    _ => (),
                }
            }
        } else {
            self.f_array[id as usize].2 = VFO_NORMAL_COLOR; 
        }
    }

    // Restore frequency
    pub fn restore_freq(&mut self) {
        // Which RX are we
        let rx = globals::get_sel_rx();
        // Retrieve and set freq
        let mut freq = self.prefs.borrow().radio.rx1.frequency;
        match rx {
            1 => {
                freq = self.prefs.borrow().radio.rx1.frequency;
                self.i_cc.lock().unwrap().cc_set_rx_tx_freq(freq);
            },
            2 => {
                freq = self.prefs.borrow().radio.rx2.frequency;
                self.i_cc.lock().unwrap().cc_set_rx2_freq(freq);
            },
            3 => {
                freq = self.prefs.borrow().radio.rx3.frequency;
                self.i_cc.lock().unwrap().cc_set_rx3_freq(freq);
            },
            _ => (),
        }
        self.frequency = freq;
    }

    // Update the frequency
    pub fn update_freq(&mut self, freq: u32) {
        self.frequency = freq;
        self.freq_updated();
    }

    // After an update set the new frequency in the system
    pub fn freq_updated(&mut self) {
        // Which RX are we
        let rx = globals::get_sel_rx();
        match rx {
            1 => {
                self.prefs.borrow_mut().radio.rx1.frequency = self.frequency;
                self.i_cc.lock().unwrap().cc_set_rx_tx_freq(self.frequency);
            },
            2 => {
                self.prefs.borrow_mut().radio.rx2.frequency = self.frequency;
                self.i_cc.lock().unwrap().cc_set_rx2_freq(self.frequency);
            },
            3 => {
                self.prefs.borrow_mut().radio.rx3.frequency = self.frequency;
                self.i_cc.lock().unwrap().cc_set_rx3_freq(self.frequency);
            },
            _ => (),
        }
        
    }

    // Get the display frequency
    pub fn get_freq(&mut self) -> u32{
        return self.frequency;
    }

    // Set the display frequency
    pub fn set_freq(&mut self) {
        // Set the digits to the new frequency
        let new_freq : String = self.frequency.to_string();
        // Need to make this a 9 digit string with leading zeros
        let num_zeros = 9 - new_freq.len();
        let mut zeros_str = String::from("");

        for _i in 0..num_zeros {
            zeros_str += "0";
        }
        let freq_str = String::from(zeros_str + &new_freq);
        // We now have a 9 digit string
        // Set each digit from the string
        self.f_array[VfoId::F100M as usize].0 = freq_str.chars().nth(0).unwrap().to_string();
        self.f_array[VfoId::F10M as usize].0 = freq_str.chars().nth(1).unwrap().to_string();
        self.f_array[VfoId::F1M as usize].0 = freq_str.chars().nth(2).unwrap().to_string();
        self.f_array[VfoId::F100K as usize].0 = freq_str.chars().nth(3).unwrap().to_string();
        self.f_array[VfoId::F10K as usize].0 = freq_str.chars().nth(4).unwrap().to_string();
        self.f_array[VfoId::F1K as usize].0 = freq_str.chars().nth(5).unwrap().to_string();
        self.f_array[VfoId::F100H as usize].0 = freq_str.chars().nth(6).unwrap().to_string();
        self.f_array[VfoId::F10H as usize].0 = freq_str.chars().nth(7).unwrap().to_string();
        self.f_array[VfoId::F1H as usize].0 = freq_str.chars().nth(8).unwrap().to_string();
    }
}