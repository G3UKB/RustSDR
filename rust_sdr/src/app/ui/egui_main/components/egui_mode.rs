/*
egui_mode.rs

Module - egui_mode
Mode sub-window

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

use crate::app::common::common_defs;
use crate::app::protocol;
use crate::app::dsp;
use crate::app::ui::egui_main::components;

use egui::{RichText, TextStyle};

use eframe::egui;

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

const MODE_NORMAL_COLOR: egui::Color32 = egui::Color32::TRANSPARENT;
const MODE_HIGHLIGHT_COLOR: egui::Color32 = egui::Color32::DARK_BLUE;

//===========================================================================================
// State for Modes
pub struct UIMode {
    _i_cc : Arc<Mutex<protocol::cc_out::CCData>>,
    _mode_pos: common_defs::EnumModePos,
    m_array: [(String, egui::Color32); 12],
    spec : Rc<RefCell<components::egui_spec::UISpec>>,
    waterfall : Rc<RefCell<components::egui_waterfall::UIWaterfall>>,
}

//===========================================================================================
// Implementation for UIApp
impl UIMode {
    pub fn new(_cc: &eframe::CreationContext<'_>, 
        i_cc : Arc<Mutex<protocol::cc_out::CCData>>, 
        spec : Rc<RefCell<components::egui_spec::UISpec>>,
        waterfall : Rc<RefCell<components::egui_waterfall::UIWaterfall>>) -> Self{

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

        // Set default mode
        dsp::dsp_interface::wdsp_set_rx_mode(0, ModeId::Lsb as i32);

        Self {
            _i_cc: i_cc,
            m_array: m_array,
            _mode_pos: common_defs::EnumModePos::Lower,
            spec: spec,
            waterfall: waterfall,
        }
    }

    //===========================================================================================
    // Populate modes window
    pub fn modes(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {

            let b = ui.button(RichText::new(&self.m_array[ModeId::Lsb as usize].0).text_style(TextStyle::Heading)
            .background_color(self.m_array[ModeId::Lsb as usize].1));
            if b.clicked() {
                self.set_mode_buttons(ModeId::Lsb as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, ModeId::Lsb as i32);
                self.spec.borrow_mut().set_mode_pos( common_defs::EnumModePos::Lower);
                self.waterfall.borrow_mut().set_mode_pos( common_defs::EnumModePos::Lower);
            }

            let b = ui.button(RichText::new(&self.m_array[ModeId::Usb as usize].0).text_style(TextStyle::Heading)
            .background_color(self.m_array[ModeId::Usb as usize].1));
            if b.clicked() {
                self.set_mode_buttons(ModeId::Usb as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, ModeId::Usb as i32);
                self.spec.borrow_mut().set_mode_pos( common_defs::EnumModePos::Upper);
                self.waterfall.borrow_mut().set_mode_pos( common_defs::EnumModePos::Upper);
            }

            let b = ui.button(RichText::new(&self.m_array[ModeId::Dsb as usize].0).text_style(TextStyle::Heading)
            .background_color(self.m_array[ModeId::Dsb as usize].1));
            if b.clicked() {
                self.set_mode_buttons(ModeId::Dsb as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, ModeId::Dsb as i32);
                self.spec.borrow_mut().set_mode_pos( common_defs::EnumModePos::Both);
                self.waterfall.borrow_mut().set_mode_pos( common_defs::EnumModePos::Both);
            }

            let b = ui.button(RichText::new(&self.m_array[ModeId::CwL as usize].0).text_style(TextStyle::Heading)
            .background_color(self.m_array[ModeId::CwL as usize].1));
            if b.clicked() {
                self.set_mode_buttons(ModeId::CwL as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, ModeId::CwL as i32);
                self.spec.borrow_mut().set_mode_pos( common_defs::EnumModePos::Lower);
                self.waterfall.borrow_mut().set_mode_pos( common_defs::EnumModePos::Lower);
            }

            let b = ui.button(RichText::new(&self.m_array[ModeId::CwU as usize].0).text_style(TextStyle::Heading)
            .background_color(self.m_array[ModeId::CwU as usize].1));
            if b.clicked() {
                self.set_mode_buttons(ModeId::CwU as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, ModeId::CwU as i32);
                self.spec.borrow_mut().set_mode_pos( common_defs::EnumModePos::Upper);
                self.waterfall.borrow_mut().set_mode_pos(common_defs::EnumModePos::Upper);
            }

            let b = ui.button(RichText::new(&self.m_array[ModeId::Fm as usize].0).text_style(TextStyle::Heading)
            .background_color(self.m_array[ModeId::Fm as usize].1));
            if b.clicked() {
                self.set_mode_buttons(ModeId::Fm as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, ModeId::Fm as i32);
                self.spec.borrow_mut().set_mode_pos( common_defs::EnumModePos::Both);
                self.waterfall.borrow_mut().set_mode_pos( common_defs::EnumModePos::Both);
            }
            let b = ui.button(RichText::new(&self.m_array[ModeId::Am as usize].0).text_style(TextStyle::Heading)
            .background_color(self.m_array[ModeId::Am as usize].1));
            if b.clicked() {
                self.set_mode_buttons(ModeId::Am as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, ModeId::Am as i32);
                self.spec.borrow_mut().set_mode_pos( common_defs::EnumModePos::Both);
                self.waterfall.borrow_mut().set_mode_pos( common_defs::EnumModePos::Both);
            }

            let b = ui.button(RichText::new(&self.m_array[ModeId::DigL as usize].0).text_style(TextStyle::Heading)
            .background_color(self.m_array[ModeId::DigL as usize].1));
            if b.clicked() {
                self.set_mode_buttons(ModeId::DigL as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, ModeId::DigL as i32);
                self.spec.borrow_mut().set_mode_pos( common_defs::EnumModePos::Lower);
                self.waterfall.borrow_mut().set_mode_pos( common_defs::EnumModePos::Lower);
            }

            let b = ui.button(RichText::new(&self.m_array[ModeId::DigU as usize].0).text_style(TextStyle::Heading)
            .background_color(self.m_array[ModeId::DigU as usize].1));
            if b.clicked() {
                self.set_mode_buttons(ModeId::DigU as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, ModeId::DigU as i32);
                self.spec.borrow_mut().set_mode_pos( common_defs::EnumModePos::Upper);
                self.waterfall.borrow_mut().set_mode_pos( common_defs::EnumModePos::Upper);
            }

            let b = ui.button(RichText::new(&self.m_array[ModeId::Spec as usize].0).text_style(TextStyle::Heading)
            .background_color(self.m_array[ModeId::Spec as usize].1));
            if b.clicked() {
                self.set_mode_buttons(ModeId::Spec as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, ModeId::Spec as i32);
                self.spec.borrow_mut().set_mode_pos( common_defs::EnumModePos::Both);
                self.waterfall.borrow_mut().set_mode_pos( common_defs::EnumModePos::Both);
            }

            let b = ui.button(RichText::new(&self.m_array[ModeId::Sam as usize].0).text_style(TextStyle::Heading)
            .background_color(self.m_array[ModeId::Sam as usize].1));
            if b.clicked() {
                self.set_mode_buttons(ModeId::Sam as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, ModeId::Sam as i32);
                self.spec.borrow_mut().set_mode_pos( common_defs::EnumModePos::Both);
                self.waterfall.borrow_mut().set_mode_pos( common_defs::EnumModePos::Both);
            }

            let b = ui.button(RichText::new(&self.m_array[ModeId::Drm as usize].0).text_style(TextStyle::Heading)
            .background_color(self.m_array[ModeId::Drm as usize].1));
            if b.clicked() {
                self.set_mode_buttons(ModeId::Drm as i32);
                dsp::dsp_interface::wdsp_set_rx_mode(0, ModeId::Drm as i32);
                self.spec.borrow_mut().set_mode_pos( common_defs::EnumModePos::Both);
                self.waterfall.borrow_mut().set_mode_pos( common_defs::EnumModePos::Both);
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
}