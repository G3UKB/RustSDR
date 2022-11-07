/*
egui_ui.rs

Module - egui_ui
User interface

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

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::sync::{Arc, Mutex};
use std::ops::Neg;

use crate::app::protocol;
use crate::app::common::messages;
use crate::app::dsp;

use eframe::egui;
use egui::{FontFamily, FontId, RichText, TextStyle};

// Mode enumerations
enum mode_id {
    LSB, 
    USB,
    DSB,
    CW_L,
    CW_U,
    FM,
    AM,
    DIG_U,
    SPEC,
    DIG_L,
    SAM,
    DRM,
}

// Filter enumerations
enum filter_id {
    F6_0KHz,
    F4_0KHz,
    F2_7KHz,
    F2_4KHz,
    F1_0KHz,
    F500Hz,
    F250Hz,
    F100Hz,
}

#[inline]
fn heading2() -> TextStyle {
    TextStyle::Name("Heading2".into())
}

#[inline]
fn heading3() -> TextStyle {
    TextStyle::Name("ContextHeading".into())
}

// Styles
fn configure_text_styles(ctx: &egui::Context) {
    use FontFamily::Proportional;

    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (TextStyle::Heading, FontId::new(25.0, Proportional)),
        (heading2(), FontId::new(22.0, Proportional)),
        (heading3(), FontId::new(19.0, Proportional)),
        (TextStyle::Body, FontId::new(16.0, Proportional)),
        (TextStyle::Monospace, FontId::new(12.0, Proportional)),
        (TextStyle::Button, FontId::new(12.0, Proportional)),
        (TextStyle::Small, FontId::new(8.0, Proportional)),
    ]
    .into();
    ctx.set_style(style);
}

// State for UIApp
pub struct UIApp {
    i_cc : Arc<Mutex<protocol::cc_out::CCDataMutex>>,
    position: f32,
    last_position: f32,
    frequency: u32,
    freq_inc: i32,
    f_100M: String,
    f_10M: String,
    f_1M: String,
    f_100K: String,
    f_10K: String,
    f_1K: String,
    f_100H: String,
    f_10H: String,
    f_1H: String,
}

// Implementation for UIApp
impl UIApp {
    pub fn new(cc: &eframe::CreationContext<'_>, i_cc : Arc<Mutex<protocol::cc_out::CCDataMutex>>) -> Self{
        configure_text_styles(&cc.egui_ctx);
        Self {
            position: 50.0,
            last_position: 50.0,
            frequency: 7100000,
            freq_inc: 0,
            f_100M: String::from("0"),
            f_10M: String::from("0"),
            f_1M: String::from("7"),
            f_100K: String::from("1"),
            f_10K: String::from("0"),
            f_1K: String::from("0"),
            f_100H: String::from("0"),
            f_10H: String::from("0"),
            f_1H: String::from("0"),
            i_cc: i_cc,
        }
    }

    // Populate modes window
    fn modes(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("LSB").clicked() {
                dsp::dsp_interface::wdsp_set_rx_mode(0, mode_id::LSB as i32);
            }
            if ui.button("USB").clicked() {
                dsp::dsp_interface::wdsp_set_rx_mode(0, mode_id::USB as i32);
            }
            if ui.button("AM").clicked() {
                dsp::dsp_interface::wdsp_set_rx_mode(0, mode_id::AM as i32);
            }
            if ui.button("FM").clicked() {
                dsp::dsp_interface::wdsp_set_rx_mode(0, mode_id::FM as i32);
            }
            if ui.button("DIG_L").clicked() {
                dsp::dsp_interface::wdsp_set_rx_mode(0, mode_id::DIG_L as i32);
            }
        });
    }

    // Populate filters window
    fn filters(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("6K0").clicked() {
                dsp::dsp_interface::wdsp_set_rx_filter(0, filter_id::F6_0KHz as i32);
            }
            if ui.button("4K0").clicked() {
                dsp::dsp_interface::wdsp_set_rx_filter(0, filter_id::F4_0KHz as i32);
            }
            if ui.button("1K0").clicked() {
                dsp::dsp_interface::wdsp_set_rx_filter(0, filter_id::F1_0KHz as i32);
            }
            if ui.button("500H").clicked() {
                dsp::dsp_interface::wdsp_set_rx_filter(0, filter_id::F500Hz as i32);
            }
            if ui.button("100H").clicked() {
                dsp::dsp_interface::wdsp_set_rx_filter(0, filter_id::F100Hz as i32);
            }
        });
    }

    // Populate VFO window
    fn vfo(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(14.0,5.0);
            
            let f_100M = ui.label(RichText::new(&self.f_100M).text_style(heading2())
            .size(30.0)
            .strong());
            self.scroll_if(ui, f_100M.rect, 100000000);
            let f_10M = ui.label(RichText::new(&self.f_10M).text_style(heading2()).strong()
            .text_style(heading2())
            .size(30.0)
            .strong());
            self.scroll_if(ui, f_10M.rect, 10000000);
            let f_1M = ui.label(RichText::new(&self.f_1M).text_style(heading2()).strong()
            .text_style(heading2())
            .size(30.0)
            .strong());
            self.scroll_if(ui, f_1M.rect, 1000000);
            ui.label(RichText::new("-").text_style(heading2()).strong()
            .text_style(heading2())
            .size(30.0)
            .strong());
            let f_100K = ui.label(RichText::new(&self.f_100K).text_style(heading2()).strong()
            .text_style(heading2())
            .size(30.0)
            .strong());
            self.scroll_if(ui, f_100K.rect, 100000);
            let f_10K = ui.label(RichText::new(&self.f_10K).text_style(heading2()).strong()
            .text_style(heading2())
            .size(30.0)
            .strong());
            self.scroll_if(ui, f_10K.rect, 10000);
            let f_1K = ui.label(RichText::new(&self.f_1K).text_style(heading2()).strong()
            .text_style(heading2())
            .size(30.0)
            .strong());
            self.scroll_if(ui, f_1K.rect, 1000);
            ui.label(RichText::new("-").text_style(heading2()).strong()
            .text_style(heading2())
            .strong()
            .size(30.0)
            .strong());
            let f_100H = ui.label(RichText::new(&self.f_100H).text_style(heading2()).strong()
            .text_style(heading2())
            .size(30.0)
            .strong());
            self.scroll_if(ui, f_100H.rect, 100);
            let f_10H = ui.label(RichText::new(&self.f_10H).text_style(heading2()).strong()
            .text_style(heading2())
            .strong()
            .size(30.0)
            .strong());
            self.scroll_if(ui, f_10H.rect, 10);
            let f_1H = ui.label(RichText::new(&self.f_1H).text_style(heading2()).strong()
            .text_style(heading2())
            .size(30.0)
            .strong());
            self.scroll_if(ui, f_1H.rect, 1);
        });
    }

    // If within the rectangle of a digit and the mouse wheel is being scrolled
    fn scroll_if(&mut self, ui: &mut egui::Ui, rect: egui::Rect, inc_or_dec: i32) {
        if ui.rect_contains_pointer(rect) {
            let e = &ui.ctx().input().events;
            if e.len() > 0 {
                match &e[0] {
                    egui::Event::Scroll(v) => {
                        let mut dir = inc_or_dec;
                        if v[1] < 0.0 {
                            dir = dir.neg();
                        }
                        self.frequency = (self.frequency as i32 + dir) as u32;
                        self.set_freq();
                        self.i_cc.lock().unwrap().cc_set_rx_tx_freq(self.frequency);
                    }
                    _ => (),
                }
            }
        } 
    }

    // Set the display frequency
    fn set_freq(&mut self) {
        // Set the digits to the new frequency
        let new_freq : String = self.frequency.to_string();
        // Need to make this a 9 digit string with leading zeros
        let num_zeros = 9 - new_freq.len();
        let mut zeros_str = String::from("");

        for _i in 0..num_zeros {
            zeros_str += "0";
        }
        let mut freq_str = String::from(zeros_str + &new_freq);
        // We now have a 9 digit string
        // Set each digit from the string
        self.f_100M = freq_str.chars().nth(0).unwrap().to_string();
        self.f_10M = freq_str.chars().nth(1).unwrap().to_string();
        self.f_1M = freq_str.chars().nth(2).unwrap().to_string();
        self.f_100K = freq_str.chars().nth(3).unwrap().to_string();
        self.f_10K = freq_str.chars().nth(4).unwrap().to_string();
        self.f_1K = freq_str.chars().nth(5).unwrap().to_string();
        self.f_100H = freq_str.chars().nth(6).unwrap().to_string();
        self.f_10H = freq_str.chars().nth(7).unwrap().to_string();
        self.f_1H = freq_str.chars().nth(8).unwrap().to_string();
    }
}

// Create a window for each element in the UI.
impl eframe::App for UIApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::Window::new("Modes").show(ctx, |ui| {
            self.modes(ui);
        });
        egui::Window::new("Filters").show(ctx, |ui| {
            self.filters(ui);
        });
        egui::Window::new("VFO").show(ctx, |ui| {
            self.vfo(ui);
        });
    }
}
