/*
egui_spec.rs

Module - egui_spec
Spectrum sub-window

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

use crate::app::protocol;
use crate::app::common::common_defs;
use crate::app::ui::egui_main::components;
use crate::app::dsp;

use egui::{Color32, Pos2, pos2, emath};

use eframe::egui;

// Graphing constants
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

//===========================================================================================
// State for spectrum
pub struct UISpec {
    // Parameters
    i_cc : Arc<Mutex<protocol::cc_out::CCData>>,
    vfo : Rc<RefCell<components::egui_vfo::UIVfo>>,
    out_real: [f32; (common_defs::DSP_BLK_SZ ) as usize],

    // Spec
    frequency: u32,
    filter_width: i32,
    mode_pos: common_defs::EnumModePos, 
    disp_width: i32,
    mouse_pos: Pos2,
    freq_at_ptr: f32,
    draw_at_ptr: bool,

    // Waterfall
    last_disp_width: i32,
    image_data: Vec<Color32>,
    count: u32,
    image_height: i32,
    color_1: Color32,
    color_2: Color32,
    color_3: Color32,
    color_4: Color32,
    color_5: Color32,
    color_6: Color32,
    color_7: Color32,
    color_8: Color32,
}

//===========================================================================================
// Implementation for UIApp
impl UISpec {
    pub fn new(_cc: &eframe::CreationContext<'_>, i_cc : Arc<Mutex<protocol::cc_out::CCData>>, vfo : Rc<RefCell<components::egui_vfo::UIVfo>>) -> Self{

        Self {
            i_cc: i_cc,
            vfo: vfo,
            out_real: [0.0; (common_defs::DSP_BLK_SZ ) as usize],

            frequency: 7100000,
            disp_width: 300,
            mode_pos: common_defs::EnumModePos::Lower,
            filter_width: 2400,
            mouse_pos: pos2(0.0,0.0),
            freq_at_ptr: 7.1,
            draw_at_ptr: false,

            last_disp_width: 300,
            image_height: 100,
            image_data: vec![Color32::TRANSPARENT; 30000],
            count: 0,
            color_1: Color32::from_rgb(0, 0, 51),
            color_2: Color32::from_rgb(25, 0, 76),
            color_3: Color32::from_rgb(51, 0, 102),
            color_4: Color32::from_rgb(76, 0, 127),
            color_5: Color32::from_rgb(127, 0, 102),
            color_6: Color32::from_rgb(178, 0, 127),
            color_7: Color32::from_rgb(220, 0, 25),
            color_8: Color32::from_rgb(0255, 0, 0),
        }
    }

    pub fn set_mode_pos(&mut self, pos: common_defs::EnumModePos) {
        self.mode_pos = pos;
    }

    pub fn set_filt_width(&mut self, width: i32) {
        self.filter_width = width;
    }

    pub fn spectrum(&mut self, ui: &mut egui::Ui, out_real: &mut [f32; (common_defs::DSP_BLK_SZ ) as usize]) {
        self.out_real = *out_real;

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
            // Get the current frequency
            self.frequency = self.vfo.borrow_mut().get_freq();
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
            if self.mode_pos == common_defs::EnumModePos::Lower {
                pos_top_left = emath::pos2(rect.left() + L_MARGIN + (self.disp_width as f32/2.0) - filt_pix, rect.top() + T_MARGIN);
                pos_bottom_right = emath::pos2(rect.left() + L_MARGIN + (self.disp_width as f32/2.0), rect.top() + rect.height() - B_MARGIN);
            } else if self.mode_pos == common_defs::EnumModePos::Upper{
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
                                self.vfo.borrow_mut().update_freq(f);
                                self.vfo.borrow_mut().set_freq();
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

            // Add the waterfall display.
            if self.disp_width != self.last_disp_width {
                // Resize the image data vector
                // This vector is a linear representation of all pixel colors in the 2D waterfall display
                // It serves as the backing store to set all pixels in the image.
                self.image_data.resize((self.disp_width*self.image_height) as usize, Color32::TRANSPARENT);
                self.last_disp_width = self.disp_width;
            }
            // The vector may be newly initialised to Color32::TRANSPARENT or it may contain historical data.
            // Whichever, the process is the same. New data is added at the top for a single pixel row and existing
            // data is moved down by one row. This means the bottom row is lost.
            self.count +=1;
            if self.count % 20 == 0 {
                self.create_image_data();
            }
            let mut img = egui::ColorImage::new([self.disp_width as usize, self.image_height as usize], Color32::TRANSPARENT);
            self.wf_update(&mut img);
            let texture = egui::Context::load_texture(ui.ctx(), "wf", img, egui::TextureFilter::Linear);
            //ui.add_space(100.0);
            ui.image(texture.id(), egui::vec2(rect.width() as f32, self.image_height as f32));
        });
    }

    //==================================================================================
    // Spec helpers
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

    //==================================================================================
    // Waterfall helpers
    // Create a new image data
    fn create_image_data(&mut self) {
        // Create a new vector containing one new row of data.
        let mut new_data: Vec<Color32> = vec![Color32::TRANSPARENT; self.disp_width as usize];
        for i in 0..self.disp_width {
            let color = self.db_to_color(self.out_real[(self.disp_width - i - 1)as usize] as i32);
            new_data[i as usize] = color;
        }
        // Truncate the image_data to remove one row of data from the end.
        self.image_data.truncate(self.image_data.len() - self.disp_width as usize);
        // Append image_data to the new row of data leaving the new row at the top
        new_data.append(&mut self.image_data);
        // Set this as the new image data
        self.image_data = new_data;
    }

    // Update the waterfall image from image data
    fn wf_update(&mut self, img: &mut egui::ColorImage) {
        for y in 0..self.image_height {
            for x in L_MARGIN as i32..self.disp_width + R_MARGIN as i32  {
                img[(x as usize, y as usize)] = self.image_data[((y*self.disp_width) +x) as usize];
            }
        }
    }

    // Convert a dBM value to a colour
    fn db_to_color(&mut self, db_m: i32) -> Color32 {
        if db_m >= -160 && db_m < -135 {return self.color_1};
        if db_m >= -135 && db_m < -130 {return self.color_2};
        if db_m >= -130 && db_m < -125 {return self.color_3};
        if db_m >= -125 && db_m < -120 {return self.color_4};
        if db_m >= -120 && db_m < -115 {return self.color_5};
        if db_m >= -115 && db_m < -110 {return self.color_6};
        if db_m >= -110 && db_m < -100 {return self.color_7};
        if db_m >= -100 && db_m < 0 {return self.color_8};
        return self.color_1;
    }
}