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
use std::thread;
use std::time::Duration;

use crate::app::common::globals;
use crate::app::common::common_defs;
use crate::app::common::prefs;
use crate::app::protocol;
use crate::app::dsp;
use crate::app::udp::hw_control;

use eframe::egui;

//===========================================================================================
// State for UIApp
pub struct UIMain {
    _i_cc : Arc<Mutex<protocol::cc_out::CCData>>,
    control : components::egui_control::UIControl,
    modes : components::egui_mode::UIMode,
    filters : components::egui_filter::UIFilter,
    vfo : Rc<RefCell<components::egui_vfo::UIVfo>>,
    spec : Rc<RefCell<components::egui_spec::UISpec>>,
    out_real: [f32; (common_defs::DSP_BLK_SZ ) as usize],
    prefs: Rc<RefCell<prefs::Prefs>>,
    _hw: Rc<RefCell<hw_control::HWData>>
}

//===========================================================================================
// Implementation for UIApp
impl UIMain {
    pub fn new(cc: &eframe::CreationContext<'_>, i_cc : Arc<Mutex<protocol::cc_out::CCData>>, prefs: Rc<RefCell<prefs::Prefs>>, hw: Rc<RefCell<hw_control::HWData>>) -> Self{

        let control = components::egui_control::UIControl::new(i_cc.clone(), prefs.clone(), hw.clone());
        let vfo = Rc::new(RefCell::new(components::egui_vfo::UIVfo::new(cc, i_cc.clone(), prefs.clone())));
        let spec = Rc::new(RefCell::new(components::egui_spec::UISpec::new(cc, i_cc.clone(), vfo.clone())));
        let modes = components::egui_mode::UIMode::new(cc, i_cc.clone(), spec.clone(), prefs.clone());
        let filters = components::egui_filter::UIFilter::new(cc, i_cc.clone(), spec.clone(), prefs.clone());
        
        Self {
            _i_cc : i_cc,
            control: control,
            modes : modes,
            filters : filters,
            vfo : vfo,
            spec : spec,
            out_real: [0.0; (common_defs::DSP_BLK_SZ ) as usize],
            prefs: prefs,
            _hw: hw,
        }
    }
}

// Create a window for each element in the UI.
impl eframe::App for UIMain {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

        // Get the latest data update
        dsp::dsp_interface::wdsp_get_display_data(0, &mut self.out_real);

        let sel_rx = globals::get_sel_rx();
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut rx = String::from("");
            match sel_rx {
                1 => rx = String::from("RX-1"),
                2 => rx = String::from("RX-2"),
                3 => rx = String::from("RX-3"),
                _ => (),
            }
            ui.heading(String::from("Rust SDR Application [") + &rx + "]");
        });

        // Control window
        let x = self.prefs.borrow().windows.ctrl_x;
        let y = self.prefs.borrow().windows.ctrl_y;
        let width = self.prefs.borrow().windows.ctrl_w - 12.0;
        let height = self.prefs.borrow().windows.ctrl_h;
        let w = egui::Window::new("Control")
        .default_size(egui::vec2(width, height))
        .default_pos(egui::pos2(x,y))
        .show(ctx, |ui| {
            self.control.control(ui);
        });
        let r = w.unwrap().response.rect;
        //println!("After: {},{},{},{},",r.left(),r.top(),r.width(),r.height());
        self.prefs.borrow_mut().windows.ctrl_x = r.left();
        self.prefs.borrow_mut().windows.ctrl_y = r.top();
        self.prefs.borrow_mut().windows.ctrl_w = r.width();
        self.prefs.borrow_mut().windows.ctrl_h = r.height();

        // Modes window
        let x = self.prefs.borrow().windows.mode_x;
        let y = self.prefs.borrow().windows.mode_y;
        let width = self.prefs.borrow().windows.mode_w - 12.0;
        let height = self.prefs.borrow().windows.mode_h;
        //println!("Before: {},{},{},{},",x,y,width,height);
        let w = egui::Window::new("Modes")
        .default_size(egui::vec2(width, height))
        .default_pos(egui::pos2(x,y))
        .show(ctx, |ui| {
            self.modes.modes(ui);
        });
        let r = w.unwrap().response.rect;
        //println!("After: {},{},{},{},",r.left(),r.top(),r.width(),r.height());
        self.prefs.borrow_mut().windows.mode_x = r.left();
        self.prefs.borrow_mut().windows.mode_y = r.top();
        self.prefs.borrow_mut().windows.mode_w = r.width();
        self.prefs.borrow_mut().windows.mode_h = r.height();
        
        // Filters window
        let x1 = self.prefs.borrow().windows.filt_x;
        let y1 = self.prefs.borrow().windows.filt_y;
        let width1 = self.prefs.borrow().windows.filt_w - 12.0;
        let height1 = self.prefs.borrow().windows.filt_h;
        let w1 = egui::Window::new("Filters")
        .default_size(egui::vec2(width1, height1))
        .default_pos(egui::pos2(x1,y1))
        .show(ctx, |ui| {
            self.filters.filters(ui);
        });
        let r1 = w1.unwrap().response.rect;
        self.prefs.borrow_mut().windows.filt_x = r1.left();
        self.prefs.borrow_mut().windows.filt_y = r1.top();
        self.prefs.borrow_mut().windows.filt_w = r1.width();
        self.prefs.borrow_mut().windows.filt_h = r1.height();

        //VFO Window
        let x2 = self.prefs.borrow().windows.vfo_x;
        let y2 = self.prefs.borrow().windows.vfo_y;
        let width2 = self.prefs.borrow().windows.vfo_w;
        let height2 = self.prefs.borrow().windows.vfo_h - 35.0;
        //println!("Before: {},{},{},{},",x2,y2,width2,height2);
        let w2 = egui::Window::new("VFO")
        .default_size(egui::vec2(width2, height2))
        .default_pos(egui::pos2(x2,y2))
        .show(ctx, |ui| {
            self.vfo.borrow_mut().vfo(ui);
        });
        let r2 = w2.unwrap().response.rect;
        //println!("After: {},{},{},{},",r2.left(),r2.top(),r2.width(),r2.height());
        self.prefs.borrow_mut().windows.vfo_x = r2.left();
        self.prefs.borrow_mut().windows.vfo_y = r2.top();
        self.prefs.borrow_mut().windows.vfo_w = r2.width();
        self.prefs.borrow_mut().windows.vfo_h = r2.height();

        // Spec/Waterfall window
        let x3 = self.prefs.borrow().windows.main_x;
        let y3 = self.prefs.borrow().windows.main_y;
        let width3 = self.prefs.borrow().windows.main_w - 12.0;
        let height3 = self.prefs.borrow().windows.main_h - 4.0;
        //println!("Before: {},{},{},{},",x3,y3,width3,height3);
        let w3 = egui::Window::new("Spectrum/Waterfall")
        .default_size(egui::vec2(width3, height3))
        .default_pos(egui::pos2(x3,y3))
        .show(ctx, |ui| {
            self.spec.borrow_mut().spectrum(ui, &mut self.out_real);
        });
        let r3 = w3.unwrap().response.rect;
        //println!("After: {},{},{},{},",r3.left(),r3.top(),r3.width(),r3.height());
        self.prefs.borrow_mut().windows.main_x = r3.left();
        self.prefs.borrow_mut().windows.main_y = r3.top();
        self.prefs.borrow_mut().windows.main_w = r3.width();
        self.prefs.borrow_mut().windows.main_h = r3.height();

        // Set any new frame metrics
        let pos = frame.info().window_info.position;
        let size = frame.info().window_info.size;
        self.prefs.borrow_mut().frame.x = pos.unwrap().x;
        self.prefs.borrow_mut().frame.y = pos.unwrap().y;
        self.prefs.borrow_mut().frame.w = size.x;
        self.prefs.borrow_mut().frame.h = size.y;

        thread::sleep(Duration::from_millis(25));
    }
}

// Instantiate the one and only main window and run the event loop
pub fn ui_run(i_cc: Arc<Mutex<protocol::cc_out::CCData>>, prefs: Rc<RefCell<prefs::Prefs>>, hw: Rc<RefCell<hw_control::HWData>>) {
    
    let x = prefs.borrow().frame.x;
    let y = prefs.borrow().frame.y;
    let w = prefs.borrow().frame.w;
    let h = prefs.borrow().frame.h;
    let options = eframe::NativeOptions {
        always_on_top: false,
        maximized: false,
        decorated: true,
        drag_and_drop_support: true,
        icon_data: None,
        initial_window_pos: Option::from(egui::Pos2::new(x, y)),
        initial_window_size: Option::from(egui::Vec2::new(w, h)),
        min_window_size: None,
        max_window_size: None,
        resizable: true,
        transparent: false,
        vsync: true,
        multisampling: 0,
        depth_buffer: 0,
        stencil_buffer: 0,
        fullscreen: false,
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        renderer: eframe::Renderer::Glow,
        follow_system_theme: true,
        default_theme: eframe::Theme::Dark,
        run_and_return: true
    };
    let i_cc = i_cc.clone();
    let prefs = prefs.clone();
    let hw = hw.clone();
    eframe::run_native(
        "Rust SDR",
        options,
        Box::new(|cc| Box::new(UIMain::new(cc, i_cc, prefs, hw))),
    );

}