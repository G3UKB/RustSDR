#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

use egui::{FontFamily, FontId, RichText, TextStyle};

#[inline]
fn heading2() -> TextStyle {
    TextStyle::Name("Heading2".into())
}

#[inline]
fn heading3() -> TextStyle {
    TextStyle::Name("ContextHeading".into())
}

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

struct MyApp {
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

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_text_styles(&cc.egui_ctx);
        Self {
            position: 0.0,
            last_position: 0.0,
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
        }
    }

    fn modes(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("LSB").clicked() {
                println!("LSB");
            }
            if ui.button("USB").clicked() {
                println!("USB");
            }
            if ui.button("AM").clicked() {
                println!("AM");
            }
            if ui.button("FM").clicked() {
                println!("FM");
            }
            if ui.button("DIG").clicked() {
                println!("DIG");
            }
        });
    }

    fn filters(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("6K0").clicked() {
                println!("6K0");
            }
            if ui.button("4K0").clicked() {
                println!("4K0");
            }
            if ui.button("1K0").clicked() {
                println!("1K0");
            }
            if ui.button("500H").clicked() {
                println!("500H");
            }
            if ui.button("100H").clicked() {
                println!("100H");
            }
        });
    }

    fn vfo(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(14.0,5.0);

            ui.label(RichText::new(&self.f_100M).text_style(heading2())
            .strong()
            .size(30.0)
            .strong());
            ui.label(RichText::new(&self.f_10M).text_style(heading2()).strong()
            .text_style(heading2())
            .strong()
            .size(30.0)
            .strong());
            ui.label(RichText::new(&self.f_1M).text_style(heading2()).strong()
            .text_style(heading2())
            .strong()
            .size(30.0)
            .strong());
            ui.label(RichText::new("-").text_style(heading2()).strong()
            .text_style(heading2())
            .strong()
            .size(30.0)
            .strong());
            ui.label(RichText::new(&self.f_100K).text_style(heading2()).strong()
            .text_style(heading2())
            .strong()
            .size(30.0)
            .strong());
            ui.label(RichText::new(&self.f_10K).text_style(heading2()).strong()
            .text_style(heading2())
            .strong()
            .size(30.0)
            .strong());
            ui.label(RichText::new(&self.f_1K).text_style(heading2()).strong()
            .text_style(heading2())
            .strong()
            .size(30.0)
            .strong());
            ui.label(RichText::new("-").text_style(heading2()).strong()
            .text_style(heading2())
            .strong()
            .size(30.0)
            .strong());
            ui.label(RichText::new(&self.f_100H).text_style(heading2()).strong()
            .text_style(heading2())
            .strong()
            .size(30.0)
            .strong());
            ui.label(RichText::new(&self.f_10H).text_style(heading2()).strong()
            .text_style(heading2())
            .strong()
            .size(30.0)
            .strong());
            ui.label(RichText::new(&self.f_1H).text_style(heading2()).strong()
            .text_style(heading2())
            .strong()
            .size(30.0)
            .strong());
        });
        ui.horizontal(|ui| {
            ui.style_mut().spacing.button_padding = egui::vec2(6.0, 5.0);
            if ui.button("^").clicked() {
                self.freq_inc = 100000000;
            };
            if ui.button("^").clicked() {
                self.freq_inc = 10000000;
            };
            if ui.button("^").clicked() {
                self.freq_inc = 1000000;
            };
            ui.add_space(30.0);
            if ui.button("^").clicked() {
                self.freq_inc = 100000;
            };
            if ui.button("^").clicked() {
                self.freq_inc = 10000;
            };
            if ui.button("^").clicked() {
                self.freq_inc = 1000;
            };
            ui.add_space(30.0);
            if ui.button("^").clicked() {
                self.freq_inc = 100;
            };
            if ui.button("^").clicked() {
                self.freq_inc = 10;
            };
            if ui.button("^").clicked() {
                self.freq_inc = 1;
            };
        });
        ui.style_mut().spacing.slider_width = 300.0;
        self.last_position = self.position;
        ui.add(egui::Slider::new(&mut self.position, -100.0..=100.0)
            .show_value(true)
        );
        let mut inc_or_dec: f32 = 0.0;
        if self.position > self.last_position {
            inc_or_dec = (self.position - self.last_position)*self.freq_inc as f32;
            self.frequency = self.frequency + inc_or_dec as u32;
        } else {
            inc_or_dec = (self.last_position - self.position)*self.freq_inc as f32;
            self.frequency = self.frequency - inc_or_dec as u32;
        }
    }
}

impl eframe::App for MyApp {
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

fn main() {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "egui example: global font style",
        options,
        Box::new(|cc| Box::new(MyApp::new(cc))),
    );
}
