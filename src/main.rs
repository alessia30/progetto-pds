#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use screenshots::Screen;
use std::time::Instant;

use egui::{ Key, Modifiers, ModifierNames, KeyboardShortcut };

fn main() -> Result<(), eframe::Error> {


    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(400.0, 240.0)),
        ..Default::default()
    };
    let catture = vec!["Rettangolo", "Schermo intero", "\u{1F5D4} Finestra","Mano libera"];
    let ritardi = vec!["Nessun ritardo", "3 secondi", "5 secondi","10 secondi"];
    let mut cattura=0;
    let mut ritardo=0;
    

    // Variabili per la shortcut
    let mut my_shortcut = KeyboardShortcut {
        modifiers: Modifiers::default(), // Imposta i modificatori desiderati
        key: Key::A, // Imposta la chiave desiderata
    };
    let is_mac = true; // Sostituisci con `true` se sei su macOS, altrimenti `false`

    let mut new_shortcut = KeyboardShortcut {
        modifiers: my_shortcut.modifiers,
        key: my_shortcut.key,
    };

    let mut is_shortcut_modal_open = false;
    // Fine variabili per la shortcut


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
                
                // Bottone "Modifica Shortcut"
                if ui.add(egui::ImageButton::new(egui::include_image!("../img/settings.png"))).on_hover_text("Modifica shortcut").clicked() {
                    is_shortcut_modal_open = true;
                }
            });

            // --- Gestione della window per la modifica della shortcut ---
            if is_shortcut_modal_open {
                egui::Window::new("Modifica shortcut").show(&ctx, |ui| {
                    // Qui puoi rilevare gli eventi di input e aggiornare la variabile key
                    for event in ui.input(|i| i.events.clone()) {
                        match event {
                            egui::Event::Key { modifiers: event_modifiers, key: event_key, pressed, .. } => {
                                println!("pressed {:}", pressed);
                                println!("event_modifiers {:?}", event_modifiers);
                                println!("event_key {:}", event_key.name());
                                if pressed {
                                    // Solo se il tasto è stato premuto
                                    new_shortcut.modifiers = event_modifiers;
                                    new_shortcut.key = event_key;
                                }
                            }
                            _ => {}
                        }
                    }
                    ui.label(&format!("Hai premuto: {:?}", new_shortcut.format(&ModifierNames {
                        is_short: false,
                        alt: "Alt",
                        ctrl: "Ctrl",
                        shift: "Shift",
                        mac_cmd: "Cmd",
                        mac_alt: "Option",
                        concat: "+",
                    }, is_mac)));
                    if ui.button("Salva").clicked() {
                        // Salva le modifiche
                        my_shortcut.modifiers = new_shortcut.modifiers;
                        my_shortcut.key = new_shortcut.key;
                        is_shortcut_modal_open = false; // Chiudi la finestra
                    }
                    
                    if ui.button("Chiudi").clicked() {
                        is_shortcut_modal_open = false; // Chiudi la finestra
                    }
                });
            }

            // Se la shortcut viene premuta... -> da sostituire con l'azione di cattura
            if ctx.input(|i| i.clone().consume_shortcut(&my_shortcut)) {
                // Esegui azioni basate sulla copia di InputState
                println!("Shortcut premuta!");
            }
        });    

        // --- PANNELLO CENTRALE ---
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(
                format!(
                    "Premi il tasto {} per catturare il contenuto dello schermo senza avviare l'Applicazione",
                    my_shortcut.format(&ModifierNames {
                        is_short: false,
                        alt: "Alt",
                        ctrl: "Ctrl",
                        shift: "Shift",
                        mac_cmd: "Cmd",
                        mac_alt: "Option",
                        concat: "+",
                    }, is_mac)
                )
            );  
        });
    })
}