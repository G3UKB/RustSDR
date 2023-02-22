/*
egui_meter.rs

Module - egui_meter
M<etering sub-window

Copyright (C) 2023 by G3UKB Bob Cowdery

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

use epaint::Color32;

use crate::app::common::common_defs;
use crate::app::dsp;

// Drawing colors
const TEXT_COLOR: Color32 = Color32::from_rgba_premultiplied(150,0,0,70);
const GRID_COLOR: Color32 = Color32::from_rgba_premultiplied(0,50,0,10);
const SIG_COLOR: Color32 = Color32::from_rgba_premultiplied(150,150,0,70);

const LEFT_MARGIN: f32 = 5.0;
const RIGHT_MARGIN: f32 = 5.0;
const TEXT_BOTTOM_MARGIN: f32 = 10.0;
const GRID_BOTTOM_MARGIN: f32 = 20.0;
const SIG_BOTTOM_MARGIN: f32 = 35.0;
const INTER_GAP: f32 = 17.0;
const FONT_SZ: f32 = 10.0;
const GRID_STROKE: f32 = 0.5;
const SIG_STROKE: f32 = 4.0;

//===========================================================================================
// State for meter
pub struct UIMeter {
    // Parameters
    legends: [String; 11],
    level: [i32; 11],
}

//===========================================================================================
// Implementation for UIMeter
impl UIMeter {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self{

        Self {
            legends: [String::from("1"), String::from("2"), String::from("3"), String::from("4"), String::from("5"), String::from("6"), String::from("7"), String::from("8"), String::from("9"), String::from("+20"), String::from("+40")],
            level: [-121, -115,-109,-103, -97,-91,-85,-79,-73,-53,-33],
        }
    }

    pub fn meter(&mut self, ui: &mut egui::Ui) {
        
        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            // Ensure repaint
            ui.ctx().request_repaint();

            // Size appropriately for meter. We don't want it to stretch.
            let desired_size = egui::vec2(200.0, 50.0);
            let (_id, rect) = ui.allocate_space(desired_size);

            // Get the painter
            let painter = ui.painter();

            // Draw legends
            for i in 0..self.legends.len() {
                painter.text(
                    egui::pos2(rect.left() + LEFT_MARGIN + (i as f32 * INTER_GAP), rect.bottom() - TEXT_BOTTOM_MARGIN),
                    egui::Align2::LEFT_CENTER,
                    &String::from(&self.legends[i]),
                    egui::FontId::new(FONT_SZ,egui::FontFamily::Proportional),
                    TEXT_COLOR,
                );
            }

            // Grid line
            painter.line_segment(
                [
                    egui::pos2(rect.left() + LEFT_MARGIN, rect.bottom() - GRID_BOTTOM_MARGIN),
                    egui::pos2(rect.right() - RIGHT_MARGIN, rect.bottom() - GRID_BOTTOM_MARGIN),
                ],
            egui::Stroke::new(GRID_STROKE, GRID_COLOR),
            );

            // Signal strength
            let sig = dsp::dsp_interface::wdsp_get_rx_meter(0, common_defs::MeterType::SAverage as i32);
            painter.line_segment(
                [
                    egui::pos2(rect.left() + LEFT_MARGIN, rect.bottom() - SIG_BOTTOM_MARGIN),
                    egui::pos2(rect.left() + self.sig_to_y(sig, (rect.width() - LEFT_MARGIN - RIGHT_MARGIN) as i32), rect.bottom() - SIG_BOTTOM_MARGIN),
                ],
            egui::Stroke::new(SIG_STROKE, SIG_COLOR),
            );
        });
    }

    // Convert signal strength in dbM to a y offset for the meter
    fn sig_to_y(&mut self, sig: f64, width: i32) -> f32{
        let mut offset = 0.0;
        let sig = sig as i32;
        for dbm_idx in 0..self.level.len()-1{
            if sig >= self.level[dbm_idx] && sig < self.level[dbm_idx+1] {
                offset = (dbm_idx as f32/self.level.len() as f32) * width as f32;
                break;
            }
        }
        return offset
    }

}