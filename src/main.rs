#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use screenshots::Screen;
use std::time::{Duration, SystemTime};
use std::thread::sleep;
use image::EncodableLayout;
use egui::{ Key, Modifiers, ModifierNames, KeyboardShortcut };

static DELAYS_VALUES:[u64;4]=[0,3,5,10];

fn main() -> Result<(), eframe::Error> {

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(400.0, 240.0)),
        ..Default::default()
    };

    eframe::run_native(
        "screen capture",
        options,
        Box::new(|cc|{
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(MyApp::new(cc))
            })
    )
}

struct MyApp<'a>{
    acquiring:bool,
    //fullscreen:bool,
    img:Option<egui::Image<'a>>,
    handle:Option<egui::TextureHandle>,
    window_size: egui::Vec2,
    counter:usize,
    my_shortcut:KeyboardShortcut,
    new_shortcut:KeyboardShortcut,
    captures:Vec<&'a str>,
    delays:Vec<&'a str>,
    capture:usize,
    delay:usize,
    is_mac:bool,
    is_shortcut_modal_open:bool,
}

impl MyApp<'_>{
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self{acquiring:false,img:None,handle:None,window_size:egui::vec2(0.0, 0.0),counter:0,
            my_shortcut:KeyboardShortcut {
                modifiers: Modifiers::default(), // Imposta i modificatori desiderati
                key: Key::A, // Imposta la chiave desiderata
            },
            new_shortcut:KeyboardShortcut {
                modifiers: Modifiers::default(),
                key: Key::A,
            },
            captures:vec!["Rettangolo", "Schermo intero", "Finestra","Mano libera"],
            delays:vec!["Nessun ritardo", "3 secondi", "5 secondi","10 secondi"],
            capture:0,delay:0,is_mac:true,is_shortcut_modal_open:false,
        }
    }
}

impl eframe::App for MyApp<'_>{
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame){   
        if !self.acquiring{
            egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {              
                ui.horizontal(|ui| {
                    if ui.button("\u{2795} Nuovo").on_hover_text("Nuova Cattura").clicked() {
                        self.acquiring=true;
                        //print!("pulsante premuto")      ;                                
                        frame.set_visible(false);                  
                        self.window_size=ctx.screen_rect().size();                                       
                    }
                    ui.add_space(60.0);
                    egui::ComboBox::from_id_source(2)
                        .width(30.0)
                        .selected_text(self.captures[self.capture])
                        .show_ui(ui, |ui| {
                            for (i, option) in self.captures.iter().enumerate() {
                                ui.selectable_value(&mut self.capture, i, option.to_string());
                            }
                        }).response.on_hover_text("Modalità di cattura");
                    ui.add_space(10.0);
                    egui::ComboBox::from_id_source(1)
                    .width(30.0)
                    .selected_text(self.delays[self.delay])
                    .show_ui(ui, |ui| {
                        for (i, option) in self.delays.iter().enumerate() {
                            ui.selectable_value(&mut self.delay, i, option.to_string());
                        }
                    }).response.on_hover_text("Ritarda cattura");
                    ui.add_space(20.0);
                    
                    // Bottone "Modifica Shortcut"
                    if ui.button("\u{2699}").on_hover_text("Modifica shortcut").clicked() {
                        self.is_shortcut_modal_open = true;
                    }
                });

                 // --- Gestione della window per la modifica della shortcut ---
                if self.is_shortcut_modal_open {
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
                                        self.new_shortcut.modifiers = event_modifiers;
                                        self.new_shortcut.key = event_key;
                                    }
                                }
                                _ => {}
                            }
                        }
                        ui.label(&format!("Hai premuto: {:?}", self.new_shortcut.format(&ModifierNames {
                            is_short: false,
                            alt: "Alt",
                            ctrl: "Ctrl",
                            shift: "Shift",
                            mac_cmd: "Cmd",
                            mac_alt: "Option",
                            concat: "+",
                        }, self.is_mac)));
                        if ui.button("Salva").clicked() {
                            // Salva le modifiche
                            self.my_shortcut.modifiers = self.new_shortcut.modifiers;
                            self.my_shortcut.key = self.new_shortcut.key;
                            self.is_shortcut_modal_open = false; // Chiudi la finestra
                        }
                        
                        if ui.button("Chiudi").clicked() {
                            self.is_shortcut_modal_open = false; // Chiudi la finestra
                        }            
                    });
                }
                // Se la shortcut viene premuta... -> da sostituire con l'azione di cattura
                if ctx.input(|i| i.clone().consume_shortcut(&self.my_shortcut)) {
                // Esegui azioni basate sulla copia di InputState
                    self.acquiring=true;
                    frame.set_visible(false);
                    println!("Shortcut premuta!");
                }         
            });
                // --- PANNELLO CENTRALE ---
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading(
                    format!(
                        "Premi il tasto {} per catturare il contenuto dello schermo senza avviare l'Applicazione",
                        self.my_shortcut.format(&ModifierNames {
                            is_short: false,
                            alt: "Alt",
                            ctrl: "Ctrl",
                            shift: "Shift",
                            mac_cmd: "Cmd",
                            mac_alt: "Option",
                            concat: "+",
                        }, self.is_mac)
                    )
                );  
            });
        }
        if self.acquiring{
            self.counter+=1;
            //print!("{}",self.counter);            
            ctx.request_repaint();
            if self.counter==20{
                let start = SystemTime::now();
                sleep(Duration::new(DELAYS_VALUES[self.delay], 0));
                match start.elapsed() {
                    Ok(_elapsed) => {
                        let screens = Screen::all().unwrap();                   
                        for screen in screens {
                            println!("capturer {screen:?}");
                            let image = screen.capture().unwrap();
                            // println!("{} {} {}",image.width(),image.height(),image.as_bytes().len());
                            let color_image=egui::ColorImage::from_rgba_unmultiplied([image.width() as usize,image.height() as usize], image.as_bytes());
                            self.handle= Some(ctx.load_texture("handle", color_image.clone(),egui::TextureOptions::LINEAR));
                            let sized_image = egui::load::SizedTexture::new(self.handle.clone().unwrap().id(), egui::vec2(color_image.size[0] as f32, color_image.size[1] as f32));
                            // println!("{} {} ",color_image.size[0],color_image.size[1]);
                            self.img = Some(egui::Image::from_texture(sized_image));
                        }
                    }
                    Err(_e) => {
                        println!("Timer error");
                    }
                }
                frame.set_visible(true);
            }
            if self.counter>=20{  
                if self.window_size != ctx.screen_rect().size(){
                    self.window_size=ctx.screen_rect().size();                  
                }
                let scale=ctx.screen_rect().size().x/self.img.clone().unwrap().size().unwrap().x;
                //let image_size = egui::vec2(self.window_size.x / scale, self.window_size.y / scale);                 
                egui::Window::new("").title_bar(false)
                        .show(&ctx, |ui| {
                           ui.add(self.img.clone().unwrap().fit_to_original_size(scale));            
                });
            }
        }
        
    }
}