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

//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::sync::{Arc, Mutex};
use std::ops::Neg;

use crate::app::protocol;
use crate::app::common::common_defs;
use crate::app::dsp;

use eframe::egui;
use egui::{FontFamily, FontId, RichText, TextStyle, Color32, Pos2, pos2, emath};

// Mode enumerations
enum ModeId {
    Lsb, 
    Usb,
    Dsb,
    CwL,
    CwU,
    Fm,
    Am,
    DigU,
    Spec,
    DigL,
    Sam,
    Drm,
}
#[derive(PartialEq)]
enum EnumModePos {
    Lower,
    Upper,
    Both,
}

// Filter enumerations
enum FilterId {
    F6_0KHz,
    F4_0KHz,
    F2_7KHz,
    F2_4KHz,
    F1_0KHz,
    F500Hz,
    F100Hz,
}

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

// Modes, Filters and VFO constants
const MHZ_SZ: f32 = 35.0;
const KHZ_SZ: f32 = 35.0;
const HZ_SZ: f32 = 25.0;
const VFO_NORMAL_COLOR: egui::Color32 = egui::Color32::TRANSPARENT;
const VFO_HIGHLIGHT_COLOR: egui::Color32 = egui::Color32::DARK_GREEN;
const MODE_NORMAL_COLOR: egui::Color32 = egui::Color32::TRANSPARENT;
const MODE_HIGHLIGHT_COLOR: egui::Color32 = egui::Color32::DARK_BLUE;
const FILT_NORMAL_COLOR: egui::Color32 = egui::Color32::TRANSPARENT;
const FILT_HIGHLIGHT_COLOR: egui::Color32 = egui::Color32::DARK_RED;

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
const SPEC_COLOR: Color32 = Color32::from_rgba_premultiplied(150,150,0,70);
const OVERLAY_COLOR: Color32 = Color32::from_rgba_premultiplied(0,30,0,10);
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
    i_cc : Arc<Mutex<protocol::cc_out::CCData>>,
    frequency: u32,
    mode_pos: EnumModePos,
    filter_width: i32,

    // VFO, mode and filter state
    f_array: [(String, f32, egui::Color32); 9],
    m_array: [(String, egui::Color32); 12],
    fi_array: [(String, egui::Color32); 8],

    // Display data
    out_real: [f32; (common_defs::DSP_BLK_SZ ) as usize],
    disp_width: i32,
    mouse_pos: Pos2,
    freq_at_ptr: f32,
    draw_at_ptr: bool,
}

//===========================================================================================
// Implementation for UIApp
impl UIApp {
    pub fn new(cc: &eframe::CreationContext<'_>, i_cc : Arc<Mutex<protocol::cc_out::CCData>>) -> Self{
        configure_text_styles(&cc.egui_ctx);

        // Create array of strings and size for VFO digits
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

        let m_array = [
           (String::from("LSB"), MODE_HIGHLIGHT_COLOR),
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
           (String::from("2K4"), FILT_HIGHLIGHT_COLOR),
           (String::from("1K0"), egui::Color32::TRANSPARENT),
           (String::from("500H"), egui::Color32::TRANSPARENT),
           (String::from("250H"), egui::Color32::TRANSPARENT),
           (String::from("100H"), egui::Color32::TRANSPARENT),
        ];
    
        // Set default mode and filter
        dsp::dsp_interface::wdsp_set_rx_mode(0, ModeId::Lsb as i32);
        dsp::dsp_interface::wdsp_set_rx_filter(0, FilterId::F2_4KHz as i32);

        Self {
            frequency: 7100000,
            i_cc: i_cc,
            f_array: f_array,
            m_array: m_array,
            fi_array: fi_array,
            out_real: [0.0; (common_defs::DSP_BLK_SZ ) as usize],
            disp_width: 300,
            mode_pos: EnumModePos::Lower,
            filter_width: 2400,
            mouse_pos: pos2(0.0,0.0),
            freq_at_ptr: 7.1,
            draw_at_ptr: false,
        }
    }

    //===========================================================================================
    // Populate modes window
    fn modes(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {

            let b = ui.button(RichText::new(&self.m_array[ModeId::Lsb as usize].0).text_style(heading3())
            .background_color(self.m_array[ModeId::Lsb as usize].1));
            if b.clicked() {
                self.set_mode_buttons(ModeId::Lsb as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, ModeId::Lsb as i32);
                self.mode_pos = EnumModePos::Lower;
            }

            let b = ui.button(RichText::new(&self.m_array[ModeId::Usb as usize].0).text_style(heading3())
            .background_color(self.m_array[ModeId::Usb as usize].1));
            if b.clicked() {
                self.set_mode_buttons(ModeId::Usb as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, ModeId::Usb as i32);
                self.mode_pos = EnumModePos::Upper;
            }

            let b = ui.button(RichText::new(&self.m_array[ModeId::Dsb as usize].0).text_style(heading3())
            .background_color(self.m_array[ModeId::Dsb as usize].1));
            if b.clicked() {
                self.set_mode_buttons(ModeId::Dsb as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, ModeId::Dsb as i32);
                self.mode_pos = EnumModePos::Both;
            }

            let b = ui.button(RichText::new(&self.m_array[ModeId::CwL as usize].0).text_style(heading3())
            .background_color(self.m_array[ModeId::CwL as usize].1));
            if b.clicked() {
                self.set_mode_buttons(ModeId::CwL as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, ModeId::CwL as i32);
                self.mode_pos = EnumModePos::Lower;
            }

            let b = ui.button(RichText::new(&self.m_array[ModeId::CwU as usize].0).text_style(heading3())
            .background_color(self.m_array[ModeId::CwU as usize].1));
            if b.clicked() {
                self.set_mode_buttons(ModeId::CwU as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, ModeId::CwU as i32);
                self.mode_pos = EnumModePos::Upper;
            }

            let b = ui.button(RichText::new(&self.m_array[ModeId::Fm as usize].0).text_style(heading3())
            .background_color(self.m_array[ModeId::Fm as usize].1));
            if b.clicked() {
                self.set_mode_buttons(ModeId::Fm as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, ModeId::Fm as i32);
                self.mode_pos = EnumModePos::Both;
            }
            let b = ui.button(RichText::new(&self.m_array[ModeId::Am as usize].0).text_style(heading3())
            .background_color(self.m_array[ModeId::Am as usize].1));
            if b.clicked() {
                self.set_mode_buttons(ModeId::Am as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, ModeId::Am as i32);
                self.mode_pos = EnumModePos::Both;
            }

            let b = ui.button(RichText::new(&self.m_array[ModeId::DigL as usize].0).text_style(heading3())
            .background_color(self.m_array[ModeId::DigL as usize].1));
            if b.clicked() {
                self.set_mode_buttons(ModeId::DigL as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, ModeId::DigL as i32);
                self.mode_pos = EnumModePos::Lower;
            }

            let b = ui.button(RichText::new(&self.m_array[ModeId::DigU as usize].0).text_style(heading3())
            .background_color(self.m_array[ModeId::DigU as usize].1));
            if b.clicked() {
                self.set_mode_buttons(ModeId::DigU as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, ModeId::DigU as i32);
                self.mode_pos = EnumModePos::Upper;
            }

            let b = ui.button(RichText::new(&self.m_array[ModeId::Spec as usize].0).text_style(heading3())
            .background_color(self.m_array[ModeId::Spec as usize].1));
            if b.clicked() {
                self.set_mode_buttons(ModeId::Spec as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, ModeId::Spec as i32);
                self.mode_pos = EnumModePos::Both;
            }

            let b = ui.button(RichText::new(&self.m_array[ModeId::Sam as usize].0).text_style(heading3())
            .background_color(self.m_array[ModeId::Sam as usize].1));
            if b.clicked() {
                self.set_mode_buttons(ModeId::Sam as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, ModeId::Sam as i32);
                self.mode_pos = EnumModePos::Both;
            }

            let b = ui.button(RichText::new(&self.m_array[ModeId::Drm as usize].0).text_style(heading3())
            .background_color(self.m_array[ModeId::Drm as usize].1));
            if b.clicked() {
                self.set_mode_buttons(ModeId::Drm as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, ModeId::Drm as i32);
                self.mode_pos = EnumModePos::Both;
            }
        });
    }
   
    // Highlight the selected button
    fn set_mode_buttons(&mut self, id: i32) {
        for i in 0..12 {
            self.m_array[i as usize].1 = MODE_NORMAL_COLOR;
        }
        self.m_array[id as usize].1 = MODE_HIGHLIGHT_COLOR;
    }

    //===========================================================================================
    // Populate filters window
    fn filters(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let b = ui.button(RichText::new(&self.fi_array[FilterId::F6_0KHz as usize].0).text_style(heading3())
            .background_color(self.fi_array[FilterId::F6_0KHz as usize].1));
            if b.clicked() {
                self.set_filter_buttons(FilterId::F6_0KHz as i32);
                dsp::dsp_interface::wdsp_set_rx_filter(0, FilterId::F6_0KHz as i32);
                self.filter_width = 6000;
            }

            let b = ui.button(RichText::new(&self.fi_array[FilterId::F4_0KHz as usize].0).text_style(heading3())
            .background_color(self.fi_array[FilterId::F4_0KHz as usize].1));
            if b.clicked() {
                self.set_filter_buttons(FilterId::F4_0KHz as i32);
                dsp::dsp_interface::wdsp_set_rx_filter(0, FilterId::F4_0KHz as i32);
                self.filter_width = 4000;
            }

            let b = ui.button(RichText::new(&self.fi_array[FilterId::F2_7KHz as usize].0).text_style(heading3())
            .background_color(self.fi_array[FilterId::F2_7KHz as usize].1));
            if b.clicked() {
                self.set_filter_buttons(FilterId::F2_7KHz as i32);
                dsp::dsp_interface::wdsp_set_rx_filter(0, FilterId::F2_7KHz as i32);
                self.filter_width = 2700;
            }

            let b = ui.button(RichText::new(&self.fi_array[FilterId::F2_4KHz as usize].0).text_style(heading3())
            .background_color(self.fi_array[FilterId::F2_4KHz as usize].1));
            if b.clicked() {
                self.set_filter_buttons(FilterId::F2_4KHz as i32);
                dsp::dsp_interface::wdsp_set_rx_filter(0, FilterId::F2_4KHz as i32);
                self.filter_width = 2400;
            }

            let b = ui.button(RichText::new(&self.fi_array[FilterId::F1_0KHz as usize].0).text_style(heading3())
            .background_color(self.fi_array[FilterId::F1_0KHz as usize].1));
            if b.clicked() {
                self.set_filter_buttons(FilterId::F1_0KHz as i32);
                dsp::dsp_interface::wdsp_set_rx_filter(0, FilterId::F1_0KHz as i32);
                self.filter_width = 1000;
            }

            let b = ui.button(RichText::new(&self.fi_array[FilterId::F500Hz as usize].0).text_style(heading3())
            .background_color(self.fi_array[FilterId::F500Hz as usize].1));
            if b.clicked() {
                self.set_filter_buttons(FilterId::F500Hz as i32);
                dsp::dsp_interface::wdsp_set_rx_filter(0, FilterId::F500Hz as i32);
                self.filter_width = 500;
            }

            let b = ui.button(RichText::new(&self.fi_array[FilterId::F100Hz as usize].0).text_style(heading3())
            .background_color(self.fi_array[FilterId::F100Hz as usize].1));
            if b.clicked() {
                self.set_filter_buttons(FilterId::F100Hz as i32);
                dsp::dsp_interface::wdsp_set_rx_filter(0, FilterId::F100Hz as i32);
                self.filter_width = 100;
            }
        });
    }

    // Highlight the selected button
    fn set_filter_buttons(&mut self, id: i32) {
        for i in 0..8 {
            self.fi_array[i as usize].1 = FILT_NORMAL_COLOR;
        }
        self.fi_array[id as usize].1 = FILT_HIGHLIGHT_COLOR;
    }

    //===========================================================================================
    // Populate VFO window
    fn vfo(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(14.0,5.0);
            
            let f_100_m = ui.label(RichText::new(&self.f_array[VfoId::F100M as usize].0).text_style(heading2())
            .size(self.f_array[VfoId::F100M as usize].1)
            .background_color(self.f_array[VfoId::F100M as usize].2)
            .strong());
            self.scroll_if(ui, VfoId::F100M, f_100_m.rect, 100000000);

            let f_10_m = ui.label(RichText::new(&self.f_array[VfoId::F10M as usize].0).text_style(heading2())
            .size(self.f_array[VfoId::F10M as usize].1)
            .background_color(self.f_array[VfoId::F10M as usize].2)
            .strong());
            self.scroll_if(ui, VfoId::F10M ,f_10_m.rect, 10000000);

            let f_1_m = ui.label(RichText::new(&self.f_array[VfoId::F1M as usize].0).text_style(heading2())
            .size(self.f_array[VfoId::F1M as usize].1)
            .background_color(self.f_array[VfoId::F1M as usize].2)
            .strong());
            self.scroll_if(ui, VfoId::F1M, f_1_m.rect, 1000000);

            ui.label(RichText::new("-").text_style(heading2()).strong()
            .size(30.0));

            let f_100_k = ui.label(RichText::new(&self.f_array[VfoId::F100K as usize].0).text_style(heading2())
            .size(self.f_array[VfoId::F100K as usize].1)
            .background_color(self.f_array[VfoId::F100K as usize].2)
            .strong());
            self.scroll_if(ui, VfoId::F100K, f_100_k.rect, 100000);

            let f_10_k = ui.label(RichText::new(&self.f_array[VfoId::F10K as usize].0).text_style(heading2())
            .size(self.f_array[VfoId::F10K as usize].1)
            .background_color(self.f_array[VfoId::F10K as usize].2)
            .strong());
            self.scroll_if(ui, VfoId::F10K, f_10_k.rect, 10000);

            let f_1_k = ui.label(RichText::new(&self.f_array[VfoId::F1K as usize].0).text_style(heading2())
            .size(self.f_array[VfoId::F1K as usize].1)
            .background_color(self.f_array[VfoId::F1K as usize].2)
            .strong());
            self.scroll_if(ui,VfoId::F1K, f_1_k.rect, 1000);

            ui.label(RichText::new("-").text_style(heading2()).strong()
            .size(30.0));

            let f_100_h = ui.label(RichText::new(&self.f_array[VfoId::F100H as usize].0).text_style(heading2())
            .size(self.f_array[VfoId::F100H as usize].1)
            .background_color(self.f_array[VfoId::F100H as usize].2)
            .strong());
            self.scroll_if(ui, VfoId::F100H, f_100_h.rect, 100);

            let f_10_h = ui.label(RichText::new(&self.f_array[VfoId::F10H as usize].0).text_style(heading2())
            .size(self.f_array[VfoId::F10H as usize].1)
            .background_color(self.f_array[VfoId::F10H as usize].2)
            .strong());
            self.scroll_if(ui, VfoId::F10H,f_10_h.rect, 10);

            let f_1_h = ui.label(RichText::new(&self.f_array[VfoId::F1H as usize].0).text_style(heading2())
            .size(self.f_array[VfoId::F1H as usize].1)
            .background_color(self.f_array[VfoId::F1H as usize].2)
            .strong());
            self.scroll_if(ui, VfoId::F1H, f_1_h.rect, 1);
        });
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
                        self.set_freq();
                        self.i_cc.lock().unwrap().cc_set_rx_tx_freq(self.frequency);
                    }
                    _ => (),
                }
            }
        } else {
            self.f_array[id as usize].2 = VFO_NORMAL_COLOR; 
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
            let db_pixels_per_div: f32 = (rect.height() - T_MARGIN - B_MARGIN) as f32 / db_divs as f32;
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
            let start_freq: i32 = self.frequency as i32 - (SPAN_FREQ / 2);
            let freq_inc = SPAN_FREQ / DIVS;
            let pixels_per_div: f32 = (rect.width() - L_MARGIN - R_MARGIN - F_X_LABEL_ADJ) as f32 / DIVS as f32;
            let mut j = start_freq;
            for i in 0..=DIVS {
                // Draw legends
                let f = ((j as f32 /1000000.0) * 1000.0).round() / 1000.0;
                let sfreq = String::from(f.to_string());
                painter.text(
                    egui::pos2(rect.left() + F_X_MARGIN + (i as f32 * pixels_per_div), rect.top() + rect.height() - B_MARGIN + X_H_LABEL_ADJ),
                    egui::Align2::LEFT_CENTER,
                    &sfreq,
                    egui::FontId::new(14.0,egui::FontFamily::Proportional),
                    TEXT_COLOR,
                );
                // Draw lines
                let mut color = GRID_COLOR;
                if i == DIVS/2 {
                    color = CENTRE_COLOR;
                }

                painter.line_segment(
                    [
                        egui::pos2(rect.left() + L_MARGIN  + (i as f32 *pixels_per_div), rect.top() + T_MARGIN),
                        egui::pos2(rect.left() + L_MARGIN  + (i as f32 *pixels_per_div), rect.top() + rect.height() - B_MARGIN),
                    ],
                    egui::Stroke::new(0.5, color),
                );
                j += freq_inc;
            }

            // Draw spectrum
            // Update dataset if available
            dsp::dsp_interface::wdsp_get_display_data(0, &mut self.out_real);
            // Update the display width if necessary
            if self.disp_width != (rect.width() - L_MARGIN + R_MARGIN) as i32 {
                self.disp_width = (rect.width() - L_MARGIN + R_MARGIN) as i32;
                dsp::dsp_interface::wdsp_update_disp(
                    0, common_defs::FFT_SZ, common_defs::WindowTypes::Rectangular as i32, 
                    common_defs::SUB_SPANS, common_defs::IN_SZ, self.disp_width, 
                    common_defs::AvMode::PanTimeAvLin as i32, common_defs::OVER_FRAMES, 
                    common_defs::SAMPLE_RATE, common_defs::FRAME_RATE);
            }
            // The array out_real contains a set of db values, one per pixel of the horizontal display area.
            // Must be painted every iteration even when not changed otherwise it will flicker
            let mut shapes = vec![];
            let end = (rect.width() - L_MARGIN + R_MARGIN) as i32; 
            let points: Vec<egui::Pos2> = (0..end)
                .map(|i| {
                    egui::pos2(rect.left() + L_MARGIN as f32 + i as f32, 
                        rect.top() + self.val_to_coord(self.out_real[(end - i - 1) as usize], rect.height()))
                })
                .collect();
            shapes.push(epaint::Shape::line(points, egui::Stroke::new(0.25, SPEC_COLOR)));
            painter.extend(shapes);
            
            // Draw filter overlay
            let pos_top_left: Pos2;
            let pos_bottom_right: Pos2;
            // Width of filter in pixels
            let filt_pix = (self.filter_width as f32/common_defs::SMPLS_48K as f32) * self.disp_width as f32;
            if self.mode_pos == EnumModePos::Lower {
                pos_top_left = emath::pos2(rect.left() + L_MARGIN + (self.disp_width as f32/2.0) - filt_pix, rect.top() + T_MARGIN);
                pos_bottom_right = emath::pos2(rect.left() + L_MARGIN + (self.disp_width as f32/2.0), rect.top() + rect.height() - B_MARGIN);
            } else if self.mode_pos == EnumModePos::Upper{
                pos_top_left = emath::pos2(rect.left() + L_MARGIN + (self.disp_width as f32/2.0), rect.top() + T_MARGIN);
                pos_bottom_right = emath::pos2(rect.left() + L_MARGIN + (self.disp_width as f32/2.0) + filt_pix, rect.top() + rect.height() - B_MARGIN);
            } else {
                pos_top_left = emath::pos2(rect.left() + L_MARGIN + (self.disp_width as f32/2.0) - filt_pix, rect.top() + T_MARGIN);
                pos_bottom_right = emath::pos2(rect.left() + L_MARGIN + (self.disp_width as f32/2.0) + filt_pix, rect.top() + rect.height() - B_MARGIN);
            }
            let r = emath::Rect::from_two_pos(pos_top_left,pos_bottom_right);
            
            painter.rect_filled(
                r,
                3.0,
                OVERLAY_COLOR,
            );

            // Draw frequency at cursor
            if ui.rect_contains_pointer(rect) {
                // Within the area
                if  self.mouse_pos.x > rect.left() + L_MARGIN &&
                    self.mouse_pos.x < rect.right() + R_MARGIN &&
                    self.mouse_pos.y > rect.top() + T_MARGIN &&
                    self.mouse_pos.y < rect.bottom() - B_MARGIN {
                    self.draw_at_ptr = true;
                } else {
                    self.draw_at_ptr = false;
                }
                let e = &ui.ctx().input().events;
                if e.len() > 0 {
                    match &e[0] {
                        egui::Event::PointerMoved(v) => {
                            self.mouse_pos = *v;
                            self.freq_at_ptr();
                        },
                        egui::Event::PointerButton { pos, button: _, pressed, modifiers: _ } => {
                            if *pressed {
                                let f = self.freq_at_click(*pos);
                                self.frequency = f;
                                self.set_freq();
                                self.i_cc.lock().unwrap().cc_set_rx_tx_freq(self.frequency);
                            }
                        }
                        _ => ()
                    }
                }
            } else {
                self.draw_at_ptr = false;
            }
            if self.draw_at_ptr {
                let mut draw_at = self.mouse_pos.x + 5.0;
                if ((self.mouse_pos.x - L_MARGIN) as i32) > self.disp_width/2 {
                    draw_at = self.mouse_pos.x - 30.0;
                }
                painter.text(
                    egui::pos2(draw_at, self.mouse_pos.y),
                    egui::Align2::LEFT_CENTER,
                    &String::from(self.freq_at_ptr.to_string()),
                    egui::FontId::new(12.0,egui::FontFamily::Proportional),
                    TEXT_COLOR,
                );
            }
        });
    }

    // Convert a dBM value to a Y coordinate
    fn val_to_coord(&mut self, val: f32, height: f32) -> f32{
        // y-coord = disp-height - ((abs(low-dBm) - abs(dBm)) * (disp-height/span_db))
        let disp_height: f32 = height - T_MARGIN - B_MARGIN;
        let y: f32 = (disp_height as i32 - (((i32::abs(LOW_DB) - i32::abs(val as i32))) * (disp_height as i32 / (i32::abs(LOW_DB) - i32::abs(HIGH_DB))))) as f32;
        return y;
    }

    // Calculate frequency at mouse pointer
    fn freq_at_ptr(&mut self) {
        let x = self.mouse_pos.x - L_MARGIN;
        let x_frac = x/self.disp_width as f32;
        self.freq_at_ptr = (common_defs::SMPLS_48K as f32 * x_frac + (self.frequency - common_defs::SMPLS_48K /2 ) as f32)/1000000.0;
        self.freq_at_ptr = (self.freq_at_ptr * 1000.0).round() / 1000.0;
    }

    // Calculate frequency at mouse pointer on click
    fn freq_at_click(&mut self, pos: Pos2) -> u32{
        let x = pos.x - L_MARGIN;
        let x_frac = x/self.disp_width as f32;
        let f = (common_defs::SMPLS_48K as f32 * x_frac + (self.frequency - common_defs::SMPLS_48K /2 ) as f32) as u32;
        return f;
    }
}

// Create a window for each element in the UI.
impl eframe::App for UIApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::Window::new("Modes")
        .auto_sized()
        .show(ctx, |ui| {
            self.modes(ui);
        });
        egui::Window::new("Filters")
        .auto_sized()
        .show(ctx, |ui| {
            self.filters(ui);
        });
        egui::Window::new("VFO")
        .auto_sized()
        .show(ctx, |ui| {
            self.vfo(ui);
        });
        egui::Window::new("Spectrum")
        .auto_sized()
        .show(ctx, |ui| {
            self.spectrum(ui);
        });
    }
}
