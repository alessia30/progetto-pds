#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use egui::Event;
use screenshots::Screen;
use std::{time::{Instant, Duration}, sync::WaitTimeoutResult, thread::sleep};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(400.0, 240.0)),
        ..Default::default()
    };
    let catture = vec!["Rettangolo", "Schermo intero", "\u{1F5D4} Finestra","Mano libera"];
    let ritardi = vec!["Nessun ritardo", "3 secondi", "5 secondi","10 secondi"];
    let mut cattura=0;
    let mut ritardo=0;
    
    //let mut capture_area = None;
    //let mut capturing = false;

    // Inizializza l'oggetto ScreenCapture
    /*let config = CaptureConfig {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
        output_format: "png".to_string(),
    };*/
   // let mut capture = screenshots::Screen::new(config);
   let mut is_selecting = false; // Indica se l'utente sta selezionando un'area
    let mut start_x = 0.0; // Coordinate di inizio del trascinamento
    let mut start_y = 0.0;

    eframe::run_simple_native("My egui App", options, move |ctx, frame| {
        egui_extras::install_image_loaders(ctx);
        egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("\u{2795} Nuovo").on_hover_text("Nuova Cattura").clicked() {                  
                    frame.set_minimized(true);
                    for event in ctx.input(|i| i.screen_rect()){
                        
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
