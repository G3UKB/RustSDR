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
    scalar: f32,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_text_styles(&cc.egui_ctx);
        Self {
            scalar: 50.0,
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

            ui.label(RichText::new("0")
            .text_style(heading2())
            .strong()
            .size(30.0)
            .strong());
            ui.label(RichText::new("0").text_style(heading2()).strong()
            .text_style(heading2())
            .strong()
            .size(30.0)
            .strong());
            ui.label(RichText::new("0").text_style(heading2()).strong()
            .text_style(heading2())
            .strong()
            .size(30.0)
            .strong());
            ui.label(RichText::new("-").text_style(heading2()).strong()
            .text_style(heading2())
            .strong()
            .size(30.0)
            .strong());
            ui.label(RichText::new("0").text_style(heading2()).strong()
            .text_style(heading2())
            .strong()
            .size(30.0)
            .strong());
            ui.label(RichText::new("0").text_style(heading2()).strong()
            .text_style(heading2())
            .strong()
            .size(30.0)
            .strong());
            ui.label(RichText::new("0").text_style(heading2()).strong()
            .text_style(heading2())
            .strong()
            .size(30.0)
            .strong());
            ui.label(RichText::new("-").text_style(heading2()).strong()
            .text_style(heading2())
            .strong()
            .size(30.0)
            .strong());
            ui.label(RichText::new("0").text_style(heading2()).strong()
            .text_style(heading2())
            .strong()
            .size(30.0)
            .strong());
            ui.label(RichText::new("0").text_style(heading2()).strong()
            .text_style(heading2())
            .strong()
            .size(30.0)
            .strong());
            ui.label(RichText::new("0").text_style(heading2()).strong()
            .text_style(heading2())
            .strong()
            .size(30.0)
            .strong());
        });
        ui.horizontal(|ui| {
            ui.style_mut().spacing.button_padding = egui::vec2(6.0, 5.0);
            ui.button("^");
            ui.button("^");
            ui.button("^");
            ui.add_space(30.0);
            ui.button("^");
            ui.button("^");
            ui.button("^");
            ui.add_space(30.0);
            ui.button("^");
            ui.button("^");
            ui.button("^");
        });
        ui.style_mut().spacing.slider_width = 300.0;
        ui.add(egui::Slider::new(&mut self.scalar, 0.0..=100.0)
            .show_value(false)
        );
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
