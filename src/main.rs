#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    //env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };

    eframe::run_simple_native("My egui App", options, move |ctx, _frame| {

        egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {


                if ui.button("+").on_hover_text("Nuova Cattura").clicked() {

                }

                if ui.button("").on_hover_text("Nuova Cattura").clicked() {

                }


            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {

        });
    })
}
