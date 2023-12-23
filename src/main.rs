#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use screenshots::Screen;
use std::time::{Duration, SystemTime};
use std::thread::sleep;
use image::EncodableLayout;
use eframe::egui::emath;
use egui::{ Key, Modifiers, ModifierNames, KeyboardShortcut, Order, PointerButton, RawInput, Vec2, Widget, Rect, Rounding, Color32, Stroke, Pos2, Ui, Sense, Context, Window, Image, Style, Visuals, LayerId, Id, TextureId, pos2, Painter };

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
pub trait Demo {
    fn name(&self) -> &'static str;
    fn show(&mut self, ctx: &egui::Context, open: &mut bool);
}

pub trait View {
    fn ui(&mut self, ui: &mut egui::Ui);
}

pub struct Painting {
    /// in 0-1 normalized coordinates
    lines: Vec<Vec<Pos2>>,
    stroke: Stroke,
}

impl Default for Painting {
    fn default() -> Self {
        Self {
            lines: Default::default(),
            stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
        }
    }
}

impl Painting {
    pub fn ui_control(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            egui::stroke_ui(ui, &mut self.stroke, "Stroke");
            ui.separator();
            if ui.button("Clear Painting").clicked() {
                self.lines.clear();
            }
        })
        .response
    }

    pub fn ui_content(&mut self, ui: &mut Ui) -> egui::Response {
        let (mut response, painter) =
            ui.allocate_painter(egui::Vec2::new(1920.0, 1080.0), Sense::drag());

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.square_proportions()),
            response.rect,
        );
        let from_screen = to_screen.inverse();

        if self.lines.is_empty() {
            self.lines.push(vec![]);
        }

        let current_line = self.lines.last_mut().unwrap();

        if let Some(pointer_pos) = response.interact_pointer_pos() {
            let canvas_pos = from_screen * pointer_pos;
            if current_line.last() != Some(&canvas_pos) {
                current_line.push(canvas_pos);
                response.mark_changed();
            }
        } else if !current_line.is_empty() {
            self.lines.push(vec![]);
            response.mark_changed();
        }

        let shapes = self
            .lines
            .iter()
            .filter(|line| line.len() >= 2)
            .map(|line| {
                let points: Vec<Pos2> = line.iter().map(|p| to_screen * *p).collect();
                egui::Shape::line(points, self.stroke)
            });

        painter.extend(shapes);

        response
    }
}

impl Demo for Painting {
    fn name(&self) -> &'static str {
        "🖊 Painting"
    }

    fn show(&mut self, ctx: &Context, open: &mut bool) {
        use View as _;
        Window::new(self.name())
            .open(open)
            .default_size(egui::emath::vec2(512.0, 512.0))
            .vscroll(false)
            .show(ctx, |ui| self.ui(ui));
    }
}

impl View for Painting {
    fn ui(&mut self, ui: &mut Ui) {
        self.ui_control(ui);
        ui.label("Paint with your mouse/touch!");
        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            self.ui_content(ui);
        });
    }
}


struct MyApp<'a>{
    acquiring:bool,
    acquired: bool,
    color_image: Option<egui::ColorImage>,
    rect:Rect,
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
    painting:Painting,
    acquiring_pen:bool,
    texture: Option<egui::TextureHandle>,
}

impl MyApp<'_>{
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self{acquiring:false,acquired:false,color_image:None,rect:Rect::NOTHING,img:None,handle:None,window_scale:0.0,counter:0,
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
            capture:0,delay:0,is_mac:true,is_shortcut_modal_open:false,start_pos:None,current_pos:None,painting:Painting::default(),acquiring_pen:false, texture:None
        }
    }
}

impl eframe::App for MyApp<'_>{
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame){   
        if !self.acquiring && !self.acquired{
            egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {              
                ui.horizontal(|ui| {
                    if ui.button("\u{2795} Nuovo").on_hover_text("Nuova Cattura").clicked() {
                        self.acquiring=true;
                        //print!("pulsante premuto")      ;                                
                        frame.set_visible(false);                  
                                                           
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
                self.acquired=false;
                let start = SystemTime::now();
                sleep(Duration::new(DELAYS_VALUES[self.delay], 0));
                match start.elapsed() {
                    Ok(_elapsed) => {
                        let screens = Screen::all().unwrap();                   
                        for screen in screens {
                            println!("capturer {screen:?}");
                            let image = screen.capture().unwrap();
                            self.color_image=Some(egui::ColorImage::from_rgba_unmultiplied([image.width() as usize,image.height() as usize], image.as_bytes()));
                            self.handle= Some(ctx.load_texture("handle", self.color_image.clone().unwrap(),egui::TextureOptions::LINEAR));
                            let sized_image = egui::load::SizedTexture::new(self.handle.clone().unwrap().id(), egui::vec2(self.color_image.clone().unwrap().size[0] as f32, self.color_image.clone().unwrap().size[1] as f32));
                            // println!("{} {} ",color_image.size[0],color_image.size[1]);
                            self.img = Some(egui::Image::from_texture(sized_image));
                        }
                    }
                    Err(_e) => {
                        println!("Timer error");
                    }
                }
                frame.set_visible(true);
                frame.set_fullscreen(true);
            }
            if self.counter>=20{  
                egui::Window::new("")
                    .title_bar(false)
                    .frame(egui::Frame{fill:egui::Color32::from_white_alpha(10), ..Default::default()})
                    .movable(false)
                    .fixed_pos(frame.info().window_info.position.unwrap())
                    .show(ctx, |ui| {
                    
                    self.img.clone().unwrap().paint_at(ui, ctx.screen_rect());
                    
                    let (response, painter) = ui.allocate_painter(ctx.screen_rect().size(),egui::Sense::click_and_drag() );                    ctx.set_cursor_icon(egui::CursorIcon::Crosshair);
                   
                    painter.rect_filled( ctx.screen_rect(), Rounding::ZERO, Color32::from_rgba_premultiplied(0, 0, 0, 130));
                    if self.acquired{ 
                        self.rect = Rect::from_two_pos(self.start_pos.unwrap(), self.current_pos.unwrap());
                        let pixels_per_point = frame.info().native_pixels_per_point;
                        self.color_image=Some(self.color_image.clone().unwrap().region(&self.rect, pixels_per_point));
                        self.handle= Some(ctx.load_texture("handle", self.color_image.clone().unwrap(),egui::TextureOptions::LINEAR));
                        let sized_image = egui::load::SizedTexture::new(self.handle.clone().unwrap().id(), egui::vec2(self.color_image.clone().unwrap().size[0] as f32, self.color_image.clone().unwrap().size[1] as f32));
                        self.img = Some(egui::Image::from_texture(sized_image));
                        self.window_scale=self.img.clone().unwrap().size().unwrap().x/(self.rect.max.x-self.rect.min.x);
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

            egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("\u{2795} Nuovo").on_hover_text("Nuova Cattura").clicked() {
                        self.acquiring=true;
                        println!("pulsante premuto");                                
                        frame.set_visible(false);
                                         
                    }
    
                    if ui.button("\u{270F}").on_hover_text("Penna").clicked() {
                       println!("Premuto");
                       self.acquiring_pen = true;
                    }

                    // if self.acquiring_pen {
                    //     //self.painting.show(ui.ctx(), &mut true);
                        self.painting.ui_control(ui);
                });
                if self.acquiring_pen {
                    //self.painting.show(ui.ctx(), &mut true);
                    // self.painting.ui_control(ui);
                    ui.label("Paint with your mouse/touch!");
                    // egui::Frame::canvas(ui.style()).show(ui, |ui| {
                    //     // if self.acquiring {
                    //         // Mostra la funzionalità di penna solo quando l'acquisizione è attiva
                    //         self.painting.ui_content(ui);
                    //     // }
                    // });
                }
            });
            egui::CentralPanel::default().show(ctx, |ui| {
                if self.acquiring_pen {
                    // let image_size = egui::vec2(self.img.clone().unwrap().size().unwrap().x / self.window_scale, self.img.clone().unwrap().size().unwrap().y / self.window_scale);  
                    //     ui.centered_and_justified(|ui| {
                        //         ui.add(self.img.clone().unwrap().shrink_to_fit().max_size(image_size))
                        //     });
                        // let screenshot = self.img.clone().unwrap();
                        let mut painter = ctx.layer_painter(LayerId::new(Order::Debug, Id::from("Painter")));
                        painter.set_clip_rect(self.rect);
                        painter.image(TextureId::from(&self.handle.clone().unwrap()), self.rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
                        self.painting.ui_content(ui);
                // }
                    /* 
                    egui::Frame::canvas(&Style{visuals:egui::style::Visuals{window_fill: egui::Color32::TRANSPARENT, ..Default::default()}, ..Default::default()}).fill(egui::Color32::TRANSPARENT).show(ui, |ui| {
                        self.painting.set_layer_id(imag)
                        self.painting.ui_content(ui);
                        let image_size = egui::vec2(self.img.clone().unwrap().size().unwrap().x / self.window_scale, self.img.clone().unwrap().size().unwrap().y / self.window_scale);  
                        ui.centered_and_justified(|ui| {
                            ui.add(self.img.clone().unwrap().shrink_to_fit().max_size(image_size))
                        });
                    });
                    */
                }
            
                if self.acquired && !self.acquiring {
                    let image_size = egui::vec2(self.img.clone().unwrap().size().unwrap().x / self.window_scale, self.img.clone().unwrap().size().unwrap().y / self.window_scale);  
                    ui.centered_and_justified(|ui| {
                        ui.add(self.img.clone().unwrap().shrink_to_fit().max_size(image_size))
                    });
                }
            });
            
        }
    }
}