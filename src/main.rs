#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use screenshots::Screen;
use std::time::{Duration, SystemTime};
use std::thread::sleep;
use image::EncodableLayout;
use egui::{ Key, Modifiers, ModifierNames, KeyboardShortcut, Order, PointerButton, RawInput, Vec2, Widget, Rect, Rounding, Color32, Stroke };

static DELAYS_VALUES:[u64;4]=[0,3,5,10];

fn main() -> Result<(), eframe::Error> {

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(500.0, 240.0)),
        centered: true,
        min_window_size: Some(egui::vec2(500.0, 240.0)),
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
    acquired: bool,
    color_image: Option<egui::ColorImage>,
    img:Option<egui::Image<'a>>,
    handle:Option<egui::TextureHandle>,
    window_scale: f32,
    counter:usize,
    my_shortcut:KeyboardShortcut,
    new_shortcut:KeyboardShortcut,
    captures:Vec<&'a str>,
    delays:Vec<&'a str>,
    capture:usize,
    delay:usize,
    is_mac:bool,
    is_shortcut_modal_open:bool,
    start_pos:Option<egui::Pos2>,
    current_pos:Option<egui::Pos2>,
    screen:usize,
}

impl MyApp<'_>{
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self{acquiring:false,acquired:false,color_image:None,img:None,handle:None,window_scale:0.0,counter:0,
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
            capture:0,delay:0,is_mac:match cc.egui_ctx.os(){ egui::os::OperatingSystem::Mac => true, _ => false },is_shortcut_modal_open:false,start_pos:None,current_pos:None,screen:1,
        }
    }

    fn top_panel(&mut self, ctx: &egui::Context,frame: &mut eframe::Frame){
        egui::TopBottomPanel::top("my_panel").exact_height(40.0).show(ctx, |ui| {        
            ui.horizontal_centered(|ui| {
                
                if ui.button(egui::RichText::new("\u{2795} Nuovo").size(14.0)).on_hover_text("Nuova Cattura").clicked() {
                    self.acquiring=true;
                    //print!("pulsante premuto");                                
                    frame.set_visible(false);                  
                                                      
                }
                ui.add_space(10.0);
                let screens = Screen::all().unwrap();
                egui::ComboBox::from_id_source(3)
                .width(30.0)
                .selected_text(egui::RichText::new(self.screen.to_string()).size(14.0))
                .show_ui(ui, |ui| {
                    for i in 1..= screens.len() {
                        ui.selectable_value(&mut self.screen, i, i.to_string());
                    }
                }).response.on_hover_text("Schermo di cattura");
                ui.add_space(60.0);
                egui::ComboBox::from_id_source(2)
                    .width(30.0)
                    .selected_text(egui::RichText::new(self.captures[self.capture]).size(14.0))
                    .show_ui(ui, |ui| {
                        for (i, option) in self.captures.iter().enumerate() {
                            ui.selectable_value(&mut self.capture, i, option.to_string());
                        }
                    }).response.on_hover_text("Modalità di cattura");
                ui.add_space(10.0);
                egui::ComboBox::from_id_source(1)
                .width(30.0)
                .selected_text(egui::RichText::new(self.delays[self.delay]).size(14.0))
                .show_ui(ui, |ui| {
                    for (i, option) in self.delays.iter().enumerate() {
                        ui.selectable_value(&mut self.delay, i, option.to_string());
                    }
                }).response.on_hover_text("Ritarda cattura");
                ui.add_space(20.0);
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Bottone "Modifica Shortcut"
                    ui.visuals_mut().button_frame = false;
                    
                    if ui.button(egui::RichText::new("\u{2699}").size(22.0)).on_hover_text("Modifica shortcut").clicked() {
                        self.is_shortcut_modal_open = true;
                    }
                    if self.acquired{
                        if ui.button(egui::RichText::new("salva").size(14.0)).on_hover_text("").clicked() {
                            
                        }
                    }
                });
                
            });
    
             // --- Gestione della window per la modifica della shortcut ---
            if self.is_shortcut_modal_open {
                egui::Window::new("Modifica shortcut").resizable(false).anchor(egui::Align2::RIGHT_TOP, egui::vec2(10.0, 45.0)).show(&ctx, |ui| {                     
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
    }
}

impl eframe::App for MyApp<'_>{
        fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame){  
        if !self.acquiring && !self.acquired{
            self.top_panel(ctx,frame);
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.centered_and_justified(|ui|{
                    ui.heading(
                    format!(
                        "Premi il tasto {} per catturare il contenuto dello schermo",
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
            });
        }
        if self.acquiring{
            self.counter+=1;
            //print!("{}",self.counter);            
            ctx.request_repaint();
            if self.counter==20{
                let screens = Screen::all().unwrap(); 
                let screen = screens[self.screen-1];  
                self.acquired=false;
                let start = SystemTime::now();
                sleep(Duration::new(DELAYS_VALUES[self.delay], 0));
                match start.elapsed() {
                    Ok(_elapsed) => {               
                        println!("capturer {screen:?}");
                        let image = screen.capture().unwrap();
                        self.color_image=Some(egui::ColorImage::from_rgba_unmultiplied([image.width() as usize,image.height() as usize], image.as_bytes()));
                        self.handle= Some(ctx.load_texture("handle", self.color_image.clone().unwrap(),egui::TextureOptions::LINEAR));
                        let sized_image = egui::load::SizedTexture::new(self.handle.clone().unwrap().id(), egui::vec2(self.color_image.clone().unwrap().size[0] as f32, self.color_image.clone().unwrap().size[1] as f32));
                        // println!("{} {} ",color_image.size[0],color_image.size[1]);
                        self.img = Some(egui::Image::from_texture(sized_image));
                        self.window_scale=ctx.pixels_per_point();                     
                    }
                    Err(_e) => {
                        println!("Timer error");
                    }
                }
                frame.set_visible(true);
                if self.capture == 1{
                    self.acquiring = false;
                    self.acquired = true;
                    self.counter=0;
                } else {
                    frame.set_window_pos(egui::Pos2::new(screen.display_info.x as f32, screen.display_info.y as f32));
                    frame.set_fullscreen(true);
                }
            }
            if self.counter>20{  
                egui::Window::new("")
                    .title_bar(false)
                    .frame(egui::Frame{fill:egui::Color32::from_white_alpha(10), ..Default::default()})
                    .movable(false)
                    .fixed_pos(frame.info().window_info.position.unwrap())
                    .show(ctx, |ui| {
                    
                    self.img.clone().unwrap().paint_at(ui, ctx.screen_rect());
                    
                  //  let screens = Screen::all().unwrap();                   

                    let (response, painter) = ui.allocate_painter(ctx.screen_rect().size(),egui::Sense::click_and_drag() );                  
                  ctx.set_cursor_icon(egui::CursorIcon::Crosshair);
                    // (response, painter) = ui.allocate_painter(egui::Vec2::new((screens[1].display_info.width+screens[0].display_info.width) as f32, screens[0].display_info.height as f32),egui::Sense::click_and_drag());
                    painter.rect_filled( ctx.screen_rect(), Rounding::ZERO, Color32::from_rgba_premultiplied(0, 0, 0, 130));
                    if self.acquired{ 
                        let rect = Rect::from_two_pos(self.start_pos.unwrap(), self.current_pos.unwrap());
                        let pixels_per_point = frame.info().native_pixels_per_point;
                        self.color_image=Some(self.color_image.clone().unwrap().region(&rect, pixels_per_point));
                        self.handle= Some(ctx.load_texture("handle", self.color_image.clone().unwrap(),egui::TextureOptions::LINEAR));
                        let sized_image = egui::load::SizedTexture::new(self.handle.clone().unwrap().id(), egui::vec2(self.color_image.clone().unwrap().size[0] as f32, self.color_image.clone().unwrap().size[1] as f32));
                        self.img = Some(egui::Image::from_texture(sized_image));
                        self.acquiring=false;
                        self.counter=0;
                        frame.set_fullscreen(false);
                    }

                    if response.drag_started_by(PointerButton::Primary){
                        self.start_pos = response.interact_pointer_pos();
                        println!("START");
                    }

                    if response.dragged_by(PointerButton::Primary){
                        println!("DRAG");
                        self.current_pos = response.interact_pointer_pos();
                        painter.rect(Rect::from_two_pos(self.start_pos.unwrap(), self.current_pos.unwrap()), Rounding::ZERO,  Color32::from_rgba_premultiplied(30, 30, 30, 30),Stroke::new(2.0, Color32::WHITE) );
                        
                    }
                    if response.drag_released_by(PointerButton::Primary){  
                       // frame.request_screenshot();
                        self.acquired=true;
                    }
                });
            }
        }
        if self.acquired && !self.acquiring{
            self.top_panel(ctx, frame);
            egui::CentralPanel::default().show(ctx, |ui| {
                let image_size = egui::vec2(self.img.clone().unwrap().size().unwrap().x / self.window_scale, self.img.clone().unwrap().size().unwrap().y / self.window_scale);  
                ui.centered_and_justified(|ui|{ui.add(self.img.clone().unwrap().shrink_to_fit().max_size(image_size))});
            });
        }
        
    }

    
}

