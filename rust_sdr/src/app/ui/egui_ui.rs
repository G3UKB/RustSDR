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

// VFO enumeration
enum vfo_id {
    f_100M,
    f_10M,
    f_1M,
    f_100K,
    f_10K,
    f_1K,
    f_100H,
    f_10H,
    f_1H,
}

// Digit sizes
const MHzSz: f32 = 35.0;
const KHzSz: f32 = 35.0;
const HzSz: f32 = 25.0;
const MHzSzGrow: f32 = 40.0;
const KHzSzGrow: f32 = 40.0;
const HzSzGrow: f32 = 30.0;
const VFONormalColor: egui::Color32 = egui::Color32::TRANSPARENT;
const VFOHighlightColor: egui::Color32 = egui::Color32::DARK_GREEN;
const ModeNormalColor: egui::Color32 = egui::Color32::TRANSPARENT;
const ModeHighlightColor: egui::Color32 = egui::Color32::DARK_BLUE;
const FiltNormalColor: egui::Color32 = egui::Color32::TRANSPARENT;
const FiltHighlightColor: egui::Color32 = egui::Color32::DARK_RED;

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

    // VFO, mode and filter state
    f_array: [(String, f32, egui::Color32); 9],
    m_array: [(String, egui::Color32); 12],
    fi_array: [(String, egui::Color32); 8],
}

// Implementation for UIApp
impl UIApp {
    pub fn new(cc: &eframe::CreationContext<'_>, i_cc : Arc<Mutex<protocol::cc_out::CCDataMutex>>) -> Self{
        configure_text_styles(&cc.egui_ctx);

        // Create array of strings and size for VFO digits
        let f_array = [
           (String::from("0"), MHzSz, egui::Color32::TRANSPARENT),
           (String::from("0"), MHzSz, egui::Color32::TRANSPARENT),
           (String::from("7"), MHzSz, egui::Color32::TRANSPARENT),
           (String::from("1"), KHzSz, egui::Color32::TRANSPARENT),
           (String::from("0"), KHzSz, egui::Color32::TRANSPARENT),
           (String::from("0"), KHzSz, egui::Color32::TRANSPARENT),
           (String::from("0"), HzSz, egui::Color32::TRANSPARENT),
           (String::from("0"), HzSz, egui::Color32::TRANSPARENT),
           (String::from("0"), HzSz, egui::Color32::TRANSPARENT), 
        ];

        let m_array = [
           (String::from("LSB"), egui::Color32::TRANSPARENT),
           (String::from("USB"), egui::Color32::TRANSPARENT),
           (String::from("DSB"), egui::Color32::TRANSPARENT),
           (String::from("CW-L"), egui::Color32::TRANSPARENT),
           (String::from("CW-U"), egui::Color32::TRANSPARENT),
           (String::from("FM"), egui::Color32::TRANSPARENT),
           (String::from("AM"), egui::Color32::TRANSPARENT),
           (String::from("DIG_U"), egui::Color32::TRANSPARENT),
           (String::from("SPEC"), egui::Color32::TRANSPARENT),
           (String::from("DIG-L"), egui::Color32::TRANSPARENT),
           (String::from("SAM"), egui::Color32::TRANSPARENT),
           (String::from("DRM"), egui::Color32::TRANSPARENT),
        ];
       
        let fi_array = [
           (String::from("6K0"), egui::Color32::TRANSPARENT),
           (String::from("4K0"), egui::Color32::TRANSPARENT),
           (String::from("2K7"), egui::Color32::TRANSPARENT),
           (String::from("2K4"), egui::Color32::TRANSPARENT),
           (String::from("1K0"), egui::Color32::TRANSPARENT),
           (String::from("500H"), egui::Color32::TRANSPARENT),
           (String::from("250H"), egui::Color32::TRANSPARENT),
           (String::from("100H"), egui::Color32::TRANSPARENT),
        ];
    
        Self {
            position: 50.0,
            last_position: 50.0,
            frequency: 7100000,
            freq_inc: 0,
            i_cc: i_cc,
            f_array: f_array,
            m_array: m_array,
            fi_array: fi_array,
        }
    }

    // Populate modes window
    fn modes(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {

            let b = ui.button(RichText::new(&self.m_array[mode_id::LSB as usize].0).text_style(heading2())
            .background_color(self.m_array[mode_id::LSB as usize].1));
            if b.clicked() {
                self.set_mode_buttons(mode_id::LSB as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, mode_id::LSB as i32);
            }

            let b = ui.button(RichText::new(&self.m_array[mode_id::USB as usize].0).text_style(heading2())
            .background_color(self.m_array[mode_id::USB as usize].1));
            if b.clicked() {
                self.set_mode_buttons(mode_id::USB as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, mode_id::USB as i32);
            }

            let b = ui.button(RichText::new(&self.m_array[mode_id::DSB as usize].0).text_style(heading2())
            .background_color(self.m_array[mode_id::DSB as usize].1));
            if b.clicked() {
                self.set_mode_buttons(mode_id::DSB as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, mode_id::DSB as i32);
            }

            let b = ui.button(RichText::new(&self.m_array[mode_id::CW_L as usize].0).text_style(heading2())
            .background_color(self.m_array[mode_id::CW_L as usize].1));
            if b.clicked() {
                self.set_mode_buttons(mode_id::CW_L as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, mode_id::CW_L as i32);
            }

            let b = ui.button(RichText::new(&self.m_array[mode_id::CW_U as usize].0).text_style(heading2())
            .background_color(self.m_array[mode_id::CW_U as usize].1));
            if b.clicked() {
                self.set_mode_buttons(mode_id::CW_U as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, mode_id::CW_U as i32);
            }

            let b = ui.button(RichText::new(&self.m_array[mode_id::FM as usize].0).text_style(heading2())
            .background_color(self.m_array[mode_id::FM as usize].1));
            if b.clicked() {
                self.set_mode_buttons(mode_id::FM as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, mode_id::FM as i32);
            }
        });
        ui.horizontal(|ui| {

            let b = ui.button(RichText::new(&self.m_array[mode_id::AM as usize].0).text_style(heading2())
            .background_color(self.m_array[mode_id::AM as usize].1));
            if b.clicked() {
                self.set_mode_buttons(mode_id::AM as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, mode_id::AM as i32);
            }

            let b = ui.button(RichText::new(&self.m_array[mode_id::DIG_L as usize].0).text_style(heading2())
            .background_color(self.m_array[mode_id::DIG_L as usize].1));
            if b.clicked() {
                self.set_mode_buttons(mode_id::DIG_L as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, mode_id::DIG_L as i32);
            }

            let b = ui.button(RichText::new(&self.m_array[mode_id::DIG_U as usize].0).text_style(heading2())
            .background_color(self.m_array[mode_id::DIG_U as usize].1));
            if b.clicked() {
                self.set_mode_buttons(mode_id::DIG_U as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, mode_id::DIG_U as i32);
            }

            let b = ui.button(RichText::new(&self.m_array[mode_id::SPEC as usize].0).text_style(heading2())
            .background_color(self.m_array[mode_id::SPEC as usize].1));
            if b.clicked() {
                self.set_mode_buttons(mode_id::SPEC as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, mode_id::SPEC as i32);
            }

            let b = ui.button(RichText::new(&self.m_array[mode_id::SAM as usize].0).text_style(heading2())
            .background_color(self.m_array[mode_id::SAM as usize].1));
            if b.clicked() {
                self.set_mode_buttons(mode_id::SAM as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, mode_id::SAM as i32);
            }

            let b = ui.button(RichText::new(&self.m_array[mode_id::DRM as usize].0).text_style(heading2())
            .background_color(self.m_array[mode_id::DRM as usize].1));
            if b.clicked() {
                self.set_mode_buttons(mode_id::DRM as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, mode_id::DRM as i32);
            }
        });
    }
   
    // Highlight the selected button
    fn set_mode_buttons(&mut self, id: i32) {
        for i in 0..12 {
            self.m_array[i as usize].1 = ModeNormalColor;
        }
        self.m_array[id as usize].1 = ModeHighlightColor;
    }

    // Populate filters window
    fn filters(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let b = ui.button(RichText::new(&self.fi_array[filter_id::F6_0KHz as usize].0).text_style(heading2())
            .background_color(self.fi_array[filter_id::F6_0KHz as usize].1));
            if b.clicked() {
                self.set_filter_buttons(filter_id::F6_0KHz as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, filter_id::F6_0KHz as i32);
            }

            let b = ui.button(RichText::new(&self.fi_array[filter_id::F4_0KHz as usize].0).text_style(heading2())
            .background_color(self.fi_array[filter_id::F4_0KHz as usize].1));
            if b.clicked() {
                self.set_filter_buttons(filter_id::F4_0KHz as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, filter_id::F4_0KHz as i32);
            }

            let b = ui.button(RichText::new(&self.fi_array[filter_id::F2_7KHz as usize].0).text_style(heading2())
            .background_color(self.fi_array[filter_id::F2_7KHz as usize].1));
            if b.clicked() {
                self.set_filter_buttons(filter_id::F2_7KHz as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, filter_id::F2_7KHz as i32);
            }

            let b = ui.button(RichText::new(&self.fi_array[filter_id::F2_4KHz as usize].0).text_style(heading2())
            .background_color(self.fi_array[filter_id::F2_4KHz as usize].1));
            if b.clicked() {
                self.set_filter_buttons(filter_id::F2_4KHz as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, filter_id::F2_4KHz as i32);
            }

            let b = ui.button(RichText::new(&self.fi_array[filter_id::F1_0KHz as usize].0).text_style(heading2())
            .background_color(self.fi_array[filter_id::F1_0KHz as usize].1));
            if b.clicked() {
                self.set_filter_buttons(filter_id::F1_0KHz as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, filter_id::F1_0KHz as i32);
            }

            let b = ui.button(RichText::new(&self.fi_array[filter_id::F500Hz as usize].0).text_style(heading2())
            .background_color(self.fi_array[filter_id::F500Hz as usize].1));
            if b.clicked() {
                self.set_filter_buttons(filter_id::F500Hz as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, filter_id::F500Hz as i32);
            }

            let b = ui.button(RichText::new(&self.fi_array[filter_id::F100Hz as usize].0).text_style(heading2())
            .background_color(self.fi_array[filter_id::F100Hz as usize].1));
            if b.clicked() {
                self.set_filter_buttons(filter_id::F100Hz as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, filter_id::F100Hz as i32);
            }
        });
    }

    // Highlight the selected button
    fn set_filter_buttons(&mut self, id: i32) {
        for i in 0..8 {
            self.fi_array[i as usize].1 = FiltNormalColor;
        }
        self.fi_array[id as usize].1 = FiltHighlightColor;
    }

    // Populate VFO window
    fn vfo(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(14.0,5.0);
            
            let f_100M = ui.label(RichText::new(&self.f_array[vfo_id::f_100M as usize].0).text_style(heading2())
            .size(self.f_array[vfo_id::f_100M as usize].1)
            .background_color(self.f_array[vfo_id::f_100M as usize].2)
            .strong());
            self.scroll_if(ui, vfo_id::f_100M, f_100M.rect, 100000000, MHzSz, MHzSzGrow);

            let f_10M = ui.label(RichText::new(&self.f_array[vfo_id::f_10M as usize].0).text_style(heading2())
            .size(self.f_array[vfo_id::f_10M as usize].1)
            .background_color(self.f_array[vfo_id::f_10M as usize].2)
            .strong());
            self.scroll_if(ui, vfo_id::f_10M ,f_10M.rect, 10000000, MHzSz, MHzSzGrow);

            let f_1M = ui.label(RichText::new(&self.f_array[vfo_id::f_1M as usize].0).text_style(heading2())
            .size(self.f_array[vfo_id::f_1M as usize].1)
            .background_color(self.f_array[vfo_id::f_1M as usize].2)
            .strong());
            self.scroll_if(ui, vfo_id::f_1M, f_1M.rect, 1000000, MHzSz, MHzSzGrow);

            ui.label(RichText::new("-").text_style(heading2()).strong()
            .size(30.0));

            let f_100K = ui.label(RichText::new(&self.f_array[vfo_id::f_100K as usize].0).text_style(heading2())
            .size(self.f_array[vfo_id::f_100K as usize].1)
            .background_color(self.f_array[vfo_id::f_100K as usize].2)
            .strong());
            self.scroll_if(ui, vfo_id::f_100K, f_100K.rect, 100000, KHzSz, KHzSzGrow);

            let f_10K = ui.label(RichText::new(&self.f_array[vfo_id::f_10K as usize].0).text_style(heading2())
            .size(self.f_array[vfo_id::f_10K as usize].1)
            .background_color(self.f_array[vfo_id::f_10K as usize].2)
            .strong());
            self.scroll_if(ui, vfo_id::f_10K, f_10K.rect, 10000, KHzSz, KHzSzGrow);

            let f_1K = ui.label(RichText::new(&self.f_array[vfo_id::f_1K as usize].0).text_style(heading2())
            .size(self.f_array[vfo_id::f_1K as usize].1)
            .background_color(self.f_array[vfo_id::f_1K as usize].2)
            .strong());
            self.scroll_if(ui,vfo_id::f_1K, f_1K.rect, 1000, KHzSz, KHzSzGrow);

            ui.label(RichText::new("-").text_style(heading2()).strong()
            .size(30.0));

            let f_100H = ui.label(RichText::new(&self.f_array[vfo_id::f_100H as usize].0).text_style(heading2())
            .size(self.f_array[vfo_id::f_100H as usize].1)
            .background_color(self.f_array[vfo_id::f_100H as usize].2)
            .strong());
            self.scroll_if(ui, vfo_id::f_100H, f_100H.rect, 100, HzSz, HzSzGrow);

            let f_10H = ui.label(RichText::new(&self.f_array[vfo_id::f_10H as usize].0).text_style(heading2())
            .size(self.f_array[vfo_id::f_10H as usize].1)
            .background_color(self.f_array[vfo_id::f_10H as usize].2)
            .strong());
            self.scroll_if(ui, vfo_id::f_10H,f_10H.rect, 10, HzSz, HzSzGrow);

            let f_1H = ui.label(RichText::new(&self.f_array[vfo_id::f_1H as usize].0).text_style(heading2())
            .size(self.f_array[vfo_id::f_1H as usize].1)
            .background_color(self.f_array[vfo_id::f_1H as usize].2)
            .strong());
            self.scroll_if(ui, vfo_id::f_1H, f_1H.rect, 1, HzSz, HzSzGrow);
        });
    }

    // If within the rectangle of a digit then grow the digit, else shrink to normal.
    // If the mouse wheel is being scrolled then scroll the digit up or down.
    fn scroll_if(&mut self, ui: &mut egui::Ui, id: vfo_id, r: egui::Rect, inc_or_dec: i32, normal: f32, grow: f32) {
        if ui.rect_contains_pointer(r) {
            //self.f_array[id as usize].1 = grow;
            self.f_array[id as usize].2 = VFOHighlightColor; 
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
        } else {
            //self.f_array[id as usize].1 = normal;
            self.f_array[id as usize].2 = VFONormalColor; 
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
        self.f_array[vfo_id::f_100M as usize].0 = freq_str.chars().nth(0).unwrap().to_string();
        self.f_array[vfo_id::f_10M as usize].0 = freq_str.chars().nth(1).unwrap().to_string();
        self.f_array[vfo_id::f_1M as usize].0 = freq_str.chars().nth(2).unwrap().to_string();
        self.f_array[vfo_id::f_100K as usize].0 = freq_str.chars().nth(3).unwrap().to_string();
        self.f_array[vfo_id::f_10K as usize].0 = freq_str.chars().nth(4).unwrap().to_string();
        self.f_array[vfo_id::f_1K as usize].0 = freq_str.chars().nth(5).unwrap().to_string();
        self.f_array[vfo_id::f_100H as usize].0 = freq_str.chars().nth(6).unwrap().to_string();
        self.f_array[vfo_id::f_10H as usize].0 = freq_str.chars().nth(7).unwrap().to_string();
        self.f_array[vfo_id::f_1H as usize].0 = freq_str.chars().nth(8).unwrap().to_string();
    }

    fn display(&mut self, ui: &mut egui::Ui) {
    }

}

// Create a window for each element in the UI.
impl eframe::App for UIApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::Window::new("Modes")
        .show(ctx, |ui| {
            self.modes(ui);
        });
        egui::Window::new("Filters")
        .show(ctx, |ui| {
            self.filters(ui);
        });
        egui::Window::new("VFO")
        .show(ctx, |ui| {
            self.vfo(ui);
        });
        egui::Window::new("Display")
        .show(ctx, |ui| {
            self.display(ui);
        });
    }
}
