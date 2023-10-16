#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use std::fmt;
use eframe::egui;
#[derive(Debug, PartialEq, Clone, Copy)]
enum MyOption {
    First,
    Second,
    Third,
}

impl fmt::Display for MyOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };

    eframe::run_simple_native("My egui App", options, move |ctx, _frame| {
        egui_extras::install_image_loaders(ctx);
        egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.add(egui::Button::image_and_text(egui::include_image!("../img/add.png"), "Nuovo")).on_hover_text("Nuova Cattura").clicked() {
                    
                }
                ui.add_space(100.0);
                let mut selected = MyOption::First;
                egui::ComboBox::from_label("")
                    .selected_text(format!("{:?}", selected.to_string()))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut selected, MyOption::First, "First");
                         ui.selectable_value(&mut selected, MyOption::Second, "Second");
                         ui.selectable_value(&mut selected, MyOption::Third, "Third");
                    }
                );
                ui.add_space(10.0);
                if ui.button("b").on_hover_text("").clicked() {

                }
                if ui.add(egui::ImageButton::new(egui::include_image!("../img/add.png"))).on_hover_text("Nuova Cattura").clicked() {
                    

                }

            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {

        });
    })
}
