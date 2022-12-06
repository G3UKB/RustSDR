/*
egui_main.rs

Module - egui_main
User interface main window and builder

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

pub mod components;

use std::sync::{Arc, Mutex};
use std::{cell::RefCell, rc::Rc};

use crate::app::common::common_defs;
use crate::app::protocol;
use crate::app::dsp;

use eframe::egui;

//===========================================================================================
// State for UIApp

pub struct UIMain {
    _i_cc : Arc<Mutex<protocol::cc_out::CCData>>,
    modes : components::egui_mode::UIMode,
    filters : components::egui_filter::UIFilter,
    vfo : Rc<RefCell<components::egui_vfo::UIVfo>>,
    spec : Rc<RefCell<components::egui_spec::UISpec>>,
    waterfall : Rc<RefCell<components::egui_waterfall::UIWaterfall>>,
    out_real: [f32; (common_defs::DSP_BLK_SZ ) as usize],
}

//===========================================================================================
// Implementation for UIApp
impl UIMain {
    pub fn new(cc: &eframe::CreationContext<'_>, i_cc : Arc<Mutex<protocol::cc_out::CCData>>) -> Self{

        
        let vfo = Rc::new(RefCell::new(components::egui_vfo::UIVfo::new(cc, i_cc.clone())));
        let spec = Rc::new(RefCell::new(components::egui_spec::UISpec::new(cc, i_cc.clone(), vfo.clone())));
        let waterfall = Rc::new(RefCell::new(components::egui_waterfall::UIWaterfall::new(cc, i_cc.clone(), vfo.clone())));
        let modes = components::egui_mode::UIMode::new(cc, i_cc.clone(), spec.clone(), waterfall.clone());
        let filters = components::egui_filter::UIFilter::new(cc, i_cc.clone(), spec.clone(), waterfall.clone());
        Self {
            _i_cc : i_cc,
            modes : modes,
            filters : filters,
            vfo : vfo,
            spec : spec,
            waterfall : waterfall,
            out_real: [0.0; (common_defs::DSP_BLK_SZ ) as usize],
        }
    }
}

// Create a window for each element in the UI.
impl eframe::App for UIMain {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        // Get the latest data update
        dsp::dsp_interface::wdsp_get_display_data(0, &mut self.out_real);

        egui::Window::new("Modes")
        .auto_sized()
        .show(ctx, |ui| {
            self.modes.modes(ui);
        });
         
        egui::Window::new("Filters")
        .auto_sized()
        .show(ctx, |ui| {
            self.filters.filters(ui);
        });
        
        egui::Window::new("VFO")
        .auto_sized()
        .show(ctx, |ui| {
            self.vfo.borrow_mut().vfo(ui);
        });

        // We pass the same dataset into spec and waterfall
        egui::Window::new("Spectrum")
        .auto_sized()
        .show(ctx, |ui| {
            self.spec.borrow_mut().spectrum(ui, &mut self.out_real);
        });

        egui::Window::new("Waterfall")
        .auto_sized()
        .show(ctx, |ui| {
            self.waterfall.borrow_mut().waterfall(ui, &mut self.out_real);
        });
        
    }
}

// Instantiate the one and only main window and run the event loop
pub fn ui_run(i_cc: Arc<Mutex<protocol::cc_out::CCData>>) {
    let options = eframe::NativeOptions::default();
    let i_cc = i_cc.clone();
    eframe::run_native(
        "Rust SDR",
        options,
        Box::new(|cc| Box::new(UIMain::new(cc, i_cc))),
    );
}