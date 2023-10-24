#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use screenshots::Screen;
use std::time::Instant;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(400.0, 240.0)),
        ..Default::default()
    };
    let catture = vec!["Rettangolo", "Schermo intero", "\u{1F5D4} Finestra","Mano libera"];
    let ritardi = vec!["Nessun ritardo", "3 secondi", "5 secondi","10 secondi"];
    let mut cattura=0;
    let mut ritardo=0;
    eframe::run_simple_native("My egui App", options, move |ctx, _frame| {
        egui_extras::install_image_loaders(ctx);
        egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("\u{2795} Nuovo").on_hover_text("Nuova Cattura").clicked() {
                    let screens = Screen::all().unwrap();
                        for screen in screens {
                            println!("capturer {screen:?}");
                        //     let mut image = screen.capture().unwrap();
                        //    image.save("target/sc.png").unwrap();
                        let image = screen.capture_area(300, 300, 300, 300).unwrap();
                        image.save("target/capture_display_with_point.png").unwrap();

                        }
                }
                ui.add_space(60.0);
                egui::ComboBox::from_id_source(2)
                    .width(30.0)
                    .selected_text(catture[cattura])
                    .show_ui(ui, |ui| {
                        for (i, option) in catture.iter().enumerate() {
                            ui.selectable_value(&mut cattura, i, option.to_string());
                        }
                    }).response.on_hover_text("Modalità di cattura");
                ui.add_space(10.0);
                egui::ComboBox::from_id_source(1)
                .width(30.0)
                .selected_text(ritardi[ritardo])
                .show_ui(ui, |ui| {
                    for (i, option) in ritardi.iter().enumerate() {
                        ui.selectable_value(&mut ritardo, i, option.to_string());
                    }
                }).response.on_hover_text("Ritarda cattura");
                ui.add_space(20.0);
                
                
                if ui.add(egui::ImageButton::new(egui::include_image!("../img/add.png"))).on_hover_text("Nuova Cattura").clicked() {
                    
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {

        });
    })
}
