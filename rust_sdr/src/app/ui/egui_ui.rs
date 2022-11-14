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
use crate::app::common::common_defs;
use crate::app::dsp;

use eframe::egui;
use egui::{Frame, FontFamily, FontId, RichText, TextStyle, Color32, Stroke, Vec2, vec2, Pos2, pos2, emath};

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

// Modes, Filters and VFO constants
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

// Display constants
const LOW_DB: i32 = -140;
const HIGH_DB: i32 = -20;
const Y_V_LABEL_ADJ: f32 = 14.0;
const X_H_LABEL_ADJ: f32 = 15.0;
const TEXT_MARGIN: f32 = 5.0;
const L_MARGIN: f32 = 35.0;
const R_MARGIN: f32 = -10.0;
const T_MARGIN: f32 = 14.0;
const B_MARGIN: f32 = 26.0;
const TEXT_COLOR: Color32 = Color32::from_rgba_premultiplied(150,0,0,70);
const GRID_COLOR: Color32 = Color32::from_rgba_premultiplied(0,50,0,10);
const SPEC_COLOR: Color32 = Color32::from_rgba_premultiplied(0,50,50,70);
const CENTRE_COLOR: Color32 = Color32::RED;
const SPAN_FREQ: i32 = 48000;
const DIVS: i32 = 6;
const F_X_MARGIN: f32 = 15.0;
const F_X_LABEL_ADJ: f32 = 20.0;

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
        (heading3(), FontId::new(16.0, Proportional)),
        (TextStyle::Body, FontId::new(16.0, Proportional)),
        (TextStyle::Monospace, FontId::new(12.0, Proportional)),
        (TextStyle::Button, FontId::new(12.0, Proportional)),
        (TextStyle::Small, FontId::new(8.0, Proportional)),
    ]
    .into();
    ctx.set_style(style);
}

//===========================================================================================
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

    // Displat data
    out_real: [f32; (common_defs::DSP_BLK_SZ ) as usize],
}

//===========================================================================================
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
            out_real: [0.0; (common_defs::DSP_BLK_SZ ) as usize],
        }
    }

    //===========================================================================================
    // Populate modes window
    fn modes(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {

            let b = ui.button(RichText::new(&self.m_array[mode_id::LSB as usize].0).text_style(heading3())
            .background_color(self.m_array[mode_id::LSB as usize].1));
            if b.clicked() {
                self.set_mode_buttons(mode_id::LSB as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, mode_id::LSB as i32);
            }

            let b = ui.button(RichText::new(&self.m_array[mode_id::USB as usize].0).text_style(heading3())
            .background_color(self.m_array[mode_id::USB as usize].1));
            if b.clicked() {
                self.set_mode_buttons(mode_id::USB as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, mode_id::USB as i32);
            }

            let b = ui.button(RichText::new(&self.m_array[mode_id::DSB as usize].0).text_style(heading3())
            .background_color(self.m_array[mode_id::DSB as usize].1));
            if b.clicked() {
                self.set_mode_buttons(mode_id::DSB as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, mode_id::DSB as i32);
            }

            let b = ui.button(RichText::new(&self.m_array[mode_id::CW_L as usize].0).text_style(heading3())
            .background_color(self.m_array[mode_id::CW_L as usize].1));
            if b.clicked() {
                self.set_mode_buttons(mode_id::CW_L as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, mode_id::CW_L as i32);
            }

            let b = ui.button(RichText::new(&self.m_array[mode_id::CW_U as usize].0).text_style(heading3())
            .background_color(self.m_array[mode_id::CW_U as usize].1));
            if b.clicked() {
                self.set_mode_buttons(mode_id::CW_U as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, mode_id::CW_U as i32);
            }

            let b = ui.button(RichText::new(&self.m_array[mode_id::FM as usize].0).text_style(heading3())
            .background_color(self.m_array[mode_id::FM as usize].1));
            if b.clicked() {
                self.set_mode_buttons(mode_id::FM as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, mode_id::FM as i32);
            }
        });
        ui.horizontal(|ui| {

            let b = ui.button(RichText::new(&self.m_array[mode_id::AM as usize].0).text_style(heading3())
            .background_color(self.m_array[mode_id::AM as usize].1));
            if b.clicked() {
                self.set_mode_buttons(mode_id::AM as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, mode_id::AM as i32);
            }

            let b = ui.button(RichText::new(&self.m_array[mode_id::DIG_L as usize].0).text_style(heading3())
            .background_color(self.m_array[mode_id::DIG_L as usize].1));
            if b.clicked() {
                self.set_mode_buttons(mode_id::DIG_L as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, mode_id::DIG_L as i32);
            }

            let b = ui.button(RichText::new(&self.m_array[mode_id::DIG_U as usize].0).text_style(heading3())
            .background_color(self.m_array[mode_id::DIG_U as usize].1));
            if b.clicked() {
                self.set_mode_buttons(mode_id::DIG_U as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, mode_id::DIG_U as i32);
            }

            let b = ui.button(RichText::new(&self.m_array[mode_id::SPEC as usize].0).text_style(heading3())
            .background_color(self.m_array[mode_id::SPEC as usize].1));
            if b.clicked() {
                self.set_mode_buttons(mode_id::SPEC as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, mode_id::SPEC as i32);
            }

            let b = ui.button(RichText::new(&self.m_array[mode_id::SAM as usize].0).text_style(heading3())
            .background_color(self.m_array[mode_id::SAM as usize].1));
            if b.clicked() {
                self.set_mode_buttons(mode_id::SAM as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, mode_id::SAM as i32);
            }

            let b = ui.button(RichText::new(&self.m_array[mode_id::DRM as usize].0).text_style(heading3())
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

    //===========================================================================================
    // Populate filters window
    fn filters(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let b = ui.button(RichText::new(&self.fi_array[filter_id::F6_0KHz as usize].0).text_style(heading3())
            .background_color(self.fi_array[filter_id::F6_0KHz as usize].1));
            if b.clicked() {
                self.set_filter_buttons(filter_id::F6_0KHz as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, filter_id::F6_0KHz as i32);
            }

            let b = ui.button(RichText::new(&self.fi_array[filter_id::F4_0KHz as usize].0).text_style(heading3())
            .background_color(self.fi_array[filter_id::F4_0KHz as usize].1));
            if b.clicked() {
                self.set_filter_buttons(filter_id::F4_0KHz as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, filter_id::F4_0KHz as i32);
            }

            let b = ui.button(RichText::new(&self.fi_array[filter_id::F2_7KHz as usize].0).text_style(heading3())
            .background_color(self.fi_array[filter_id::F2_7KHz as usize].1));
            if b.clicked() {
                self.set_filter_buttons(filter_id::F2_7KHz as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, filter_id::F2_7KHz as i32);
            }

            let b = ui.button(RichText::new(&self.fi_array[filter_id::F2_4KHz as usize].0).text_style(heading3())
            .background_color(self.fi_array[filter_id::F2_4KHz as usize].1));
            if b.clicked() {
                self.set_filter_buttons(filter_id::F2_4KHz as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, filter_id::F2_4KHz as i32);
            }

            let b = ui.button(RichText::new(&self.fi_array[filter_id::F1_0KHz as usize].0).text_style(heading3())
            .background_color(self.fi_array[filter_id::F1_0KHz as usize].1));
            if b.clicked() {
                self.set_filter_buttons(filter_id::F1_0KHz as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, filter_id::F1_0KHz as i32);
            }

            let b = ui.button(RichText::new(&self.fi_array[filter_id::F500Hz as usize].0).text_style(heading3())
            .background_color(self.fi_array[filter_id::F500Hz as usize].1));
            if b.clicked() {
                self.set_filter_buttons(filter_id::F500Hz as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, filter_id::F500Hz as i32);
            }

            let b = ui.button(RichText::new(&self.fi_array[filter_id::F100Hz as usize].0).text_style(heading3())
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

    //===========================================================================================
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

    //===========================================================================================
    // Spectrum display
    fn spectrum(&mut self, ui: &mut egui::Ui) {
        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            // Ensure repaint
            ui.ctx().request_repaint();

            // Go with the maximum available width and keep the aspect ratio constant
            let desired_size = ui.available_width() * egui::vec2(1.0, 0.3);
            let (_id, rect) = ui.allocate_space(desired_size);

            // Get the painter
            let painter = ui.painter();

            // Draw horizontal lines and legends
            // Set up the parameters
            let db_divs = (LOW_DB.abs() - HIGH_DB.abs()) / 20;
            let db_pixels_per_div: f32 = ((rect.height() - T_MARGIN - B_MARGIN) as f32 / db_divs as f32);
            let mut j = HIGH_DB;
            for i in 0..=db_divs {
                // Draw legends
                painter.text(
                    egui::pos2(rect.left() + TEXT_MARGIN, rect.top() + Y_V_LABEL_ADJ + (i as f32 * db_pixels_per_div as f32)),
                    egui::Align2::LEFT_CENTER,
                     &String::from(j.to_string()),
                    egui::FontId::new(14.0,egui::FontFamily::Proportional),
                    TEXT_COLOR,
                );
                // Draw lines
                painter.line_segment(
                    [
                        egui::pos2(rect.left() + L_MARGIN, rect.top() + T_MARGIN + (i as f32 * db_pixels_per_div as f32)),
                        egui::pos2(rect.right() + R_MARGIN, rect.top() + T_MARGIN + (i as f32 * db_pixels_per_div as f32)),
                    ],
                    egui::Stroke::new(0.5, GRID_COLOR),
                );
                j -= 20;
            }

            // Draw verticle lines and legends
            // Set up the parameters
            let freq = 7100000;
            let start_freq: i32 = freq - (SPAN_FREQ / 2);
            let freq_inc = SPAN_FREQ / DIVS;
            let pixels_per_div: f32 = (rect.width() - L_MARGIN - R_MARGIN - F_X_LABEL_ADJ) as f32 / DIVS as f32;
            let mut j = start_freq;
            for i in 0..=DIVS {
                // Draw legends
                let sfreq: String = String::from((j as f32/1000000.0).to_string());
                painter.text(
                    egui::pos2(rect.left() + F_X_MARGIN + (i as f32 * pixels_per_div), rect.top() + rect.height() - B_MARGIN + X_H_LABEL_ADJ),
                    egui::Align2::LEFT_CENTER,
                    &sfreq,
                    egui::FontId::new(14.0,egui::FontFamily::Proportional),
                    TEXT_COLOR,
                );
                // Draw lines
                painter.line_segment(
                    [
                        egui::pos2(rect.left() + L_MARGIN  + (i as f32 *pixels_per_div), rect.top() + T_MARGIN),
                        egui::pos2(rect.left() + L_MARGIN  + (i as f32 *pixels_per_div), rect.top() + rect.height() - B_MARGIN),
                    ],
                    egui::Stroke::new(0.5, GRID_COLOR),
                );
                j += freq_inc;
            }

            // Draw spectrum
            if dsp::dsp_interface::wdsp_get_display_data(0, &mut self.out_real) {
                // The array out_real contains a set of db values, one per pixel of the horizontal display area.
                let to_screen =
                egui::emath::RectTransform::from_to(egui::Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0), rect);
                let mut shapes = vec![];
                let points: Vec<egui::Pos2> = (0..=(rect.width() - L_MARGIN as f32 + R_MARGIN as f32) as i32)
                    .map(|i| {
                        //to_screen * egui::pos2(rect.left() + L_MARGIN as f32 + i as f32, rect.top() + self.out_real[i as usize])
                        //to_screen * egui::pos2(L_MARGIN as f32 + i as f32, self.out_real[i as usize])
                        egui::pos2(rect.left() + L_MARGIN as f32 + i as f32, (rect.top() + self.out_real[i as usize])/1.8)
                    })
                    .collect();
                    println!("{:?}", points);
                shapes.push(epaint::Shape::line(points, egui::Stroke::new(1.0, SPEC_COLOR)));
                painter.extend(shapes);
            }
        });
    }

    fn display(&mut self, ui: &mut egui::Ui) {
        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            ui.ctx().request_repaint();

            let desired_size = ui.available_width() * egui::vec2(1.0, 0.35);
            let (_id, rect) = ui.allocate_space(desired_size);

            let painter = ui.painter();
            painter.rect(
                rect.shrink(1.0),
                10.0,
                ui.ctx().style().visuals.window_fill(),
                egui::Stroke::new(0.5, egui::Color32::DARK_GRAY),
            );
            painter.line_segment(
                [
                    rect.left_top() + egui::vec2(2.0, rect.height()*0.5),
                    rect.right_top() + egui::vec2(-2.0, rect.height()*0.5),
                ],
                egui::Stroke::new(0.5, egui::Color32::DARK_GREEN),
            );
            let pos_top_left = emath::pos2(rect.left() + (rect.width()/2.0) - 30.0, rect.top() + 20.0);
            let pos_bottom_right = emath::pos2(rect.left() + (rect.width()/2.0) + 30.0, rect.top() + rect.height() - 20.0);
            let r = emath::Rect::from_two_pos(pos_top_left,pos_bottom_right);
            
            painter.rect_filled(
                r,
                10.0,
                egui::color::Rgba::from_luminance_alpha(0.2, 0.2),
            );
            painter.text(
                egui::pos2(rect.left() + rect.width()*0.2, rect.top() + rect.height()*0.7),
                egui::Align2::LEFT_CENTER,
                "This is some text",
                egui::FontId::new(30.0,egui::FontFamily::Proportional),
                egui::Color32::RED,
            );
        });

        // Build display on a canvas
        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            // Make sure we get repainted
            ui.ctx().request_repaint();

            let time = ui.input().time;
            let desired_size = ui.available_width() * egui::vec2(1.0, 0.35);
            let (_id, rect) = ui.allocate_space(desired_size);

            let to_screen =
                egui::emath::RectTransform::from_to(egui::Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0), rect);
        
            let mut shapes = vec![];

            for &mode in &[2, 3, 5] {
                let mode = mode as f64;
                let n = 120;
                let speed = 1.5;

                let points: Vec<egui::Pos2> = (0..=n)
                    .map(|i| {
                        let t = i as f64 / (n as f64);
                        let amp = (time * speed * mode).sin() / mode;
                        let y = amp * (t * std::f64::consts::TAU / 2.0 * mode).sin();
                        to_screen * egui::pos2(t as f32, y as f32)
                    })
                    .collect();

                let thickness = 10.0 / mode as f32;
                shapes.push(epaint::Shape::line(points, egui::Stroke::new(thickness, egui::Color32::from_black_alpha(240))));
            }
            ui.painter().extend(shapes);
        });
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
            //self.display(ui);
            self.spectrum(ui);
        });
    }
}
