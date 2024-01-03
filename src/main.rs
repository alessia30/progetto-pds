#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use screenshots::Screen;
use std::time::{Duration, SystemTime};
use std::thread::sleep;
use image::EncodableLayout;
use egui::{ Key, Modifiers, ModifierNames, KeyboardShortcut, Order, PointerButton, RawInput, Vec2, Widget, Rect, Rounding, Color32, Stroke, ImageData, TextEdit, ColorImage, Pos2 };
use arboard::Clipboard;
use std::borrow::Cow;
use std::path::PathBuf;
use std::fs::File;
use std::time::Instant;
use image::{Rgba, RgbaImage, ImageFormat, ImageBuffer, DynamicImage};
use std::path::Path;
use chrono::{Local, DateTime};

static DELAYS_VALUES:[u64;4]=[0,3,5,10];

#[derive(Debug, PartialEq)]
    enum FileExtension {
        Png,
        Jpg,
        Gif,
    }

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

pub struct Painting {
    /// in 0-1 normalized coordinates
    lines: Vec<Vec<egui::Pos2>>,
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

pub fn my_stroke_ui(ui: &mut crate::egui::Ui, stroke: &mut egui::epaint::Stroke, text: &str) {
    let egui::epaint::Stroke { width, color } = stroke;
    ui.vertical(|ui| {
        ui.add(egui::DragValue::new(width).speed(0.1).clamp_range(0.0..=5.0))
            .on_hover_text("Width");
        //ui.add(egui::color_picker::color_edit_button_srgba(color));
        ui.label(text);
        // stroke preview:
        let (_id, stroke_rect) = ui.allocate_space(ui.spacing().interact_size);
        let left = stroke_rect.left_center();
        let right = stroke_rect.right_center();
        ui.painter().line_segment([left, right], (*width, *color));
    });
}

impl Painting {
    pub fn ui_control(&mut self, ui: &mut egui::Ui) {
            my_stroke_ui(ui, &mut self.stroke, "Stroke");
            ui.separator();
            if ui.button("Clear").clicked() {
                self.lines.clear();
            }
        
    }

    pub fn ui_content(&mut self, ui: &mut egui::Ui, ctx: &egui::Context,img: egui::Image)-> egui::Response {
        
        let (mut response, mut painter) =
            ui.allocate_painter(ui.min_size(), egui::Sense::drag());
       
       
        println!("{} {}",ui.min_size().x,ui.min_size().y);
        let to_screen = egui::emath::RectTransform::from_to(
            Rect::from_min_size(egui::Pos2::ZERO, response.rect.square_proportions()),
            response.rect,
        );
        let from_screen = to_screen.inverse();
        println!("{} {}",response.rect.max.x,response.rect.max.y);
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
                let points: Vec<egui::Pos2> = line.iter().map(|p| to_screen * *p).collect();
                egui::Shape::line(points, self.stroke)
            });

        painter.extend(shapes);

        response
    }
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
    clipboard:Clipboard,
    is_saving_modal_open:bool,
    image_name:String,
    sel_extension:FileExtension,
    extension:String,
    timestamp:DateTime<Local>,
    painting:Painting,
    acquiring_pen:bool,
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
            capture:0,delay:0,is_mac:match cc.egui_ctx.os(){ egui::os::OperatingSystem::Mac => true, _ => false },is_shortcut_modal_open:false,start_pos:None,current_pos:None,screen:1,clipboard:Clipboard::new().expect("Unable to create clipboard"),is_saving_modal_open:false,image_name:String::from("Prova"), sel_extension:FileExtension::Png,extension:String::from("png"),timestamp:Local::now(),
            painting:Painting::default(),acquiring_pen:false,
        }
    }

    fn top_panel(&mut self, ctx: &egui::Context,frame: &mut eframe::Frame){
        egui::TopBottomPanel::top("my_panel").exact_height(40.0).show(ctx, |ui| {        
            ui.horizontal_centered(|ui| {

                let new_style = egui::style::WidgetVisuals {
                    weak_bg_fill: egui::Color32::from_rgb(0x29, 0x29, 0x29),
                    bg_fill: egui::Color32::from_rgb(0x29, 0x29, 0x29),
                    bg_stroke: Stroke { width: 1., color: egui::Color32::from_rgb(0x29, 0x29, 0x29) },
                    rounding: egui::Rounding { nw: 2., ne: 2., sw: 2., se: 2. },
                    fg_stroke: Stroke{ width: 1., color: egui::Color32::WHITE} ,
                    expansion: 4.,
                };

                ctx.set_visuals(egui::style::Visuals { widgets: egui::style::Widgets { 
                    noninteractive: new_style, inactive: new_style, hovered: new_style, active: new_style, open: new_style
                }, ..Default::default()});
                
                if ui.add(egui::Button::new(egui::RichText::new("\u{2795} Nuovo").size(14.0))).on_hover_text("Nuova Cattura").clicked() {
                    self.acquiring=true;
                    //print!("pulsante premuto");                                
                    frame.set_visible(false);    
                    self.timestamp = Local::now();
                    self.image_name = self.timestamp.format("Immagine %Y-%m-%d %H%M%S").to_string();    
                    println!("{}", self.image_name);                                  
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
                    
                    if ui.add(egui::Button::new(egui::RichText::new("\u{2699}").size(22.0)).frame(false)).on_hover_text("Modifica shortcut").clicked() {
                        self.is_shortcut_modal_open = true;
                    }
                    if self.acquired{
                        if ui.button(egui::RichText::new("Copia").size(14.0)).on_hover_text("").clicked() {
                            let screen_selected = self.screen;
                            let im = take_screen(self.start_pos.unwrap(), self.color_image.clone().unwrap().width(), self.color_image.clone().unwrap().height(), screen_selected).expect("Problem taking the screenshot");
                            println!("Max width: {}", (self.img.clone().unwrap().size().unwrap().x / self.window_scale) as u32);
                            println!("Im width: {}", im.width());
                            println!("Im heigth: {}", im.height());
                            if let Err(e) = self.clipboard.set_image(arboard::ImageData {width: im.width(), height: im.height(), bytes: Cow::from(im.as_raw())}) {
                                println!("Unable to copy in the clipboard: {:?}", e);
                            }
                        }
                        if ui.button(egui::RichText::new("Salva").size(14.0)).on_hover_text("").clicked() {
                            self.is_saving_modal_open = true;
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
                self.timestamp = Local::now();
                self.image_name = self.timestamp.format("Immagine %Y-%m-%d %H%M%S").to_string();    
                println!("{}", self.image_name);  
            }

            // --- Gestione della window per le opzioni di salvataggio ---     
            if self.is_saving_modal_open {
                egui::Window::new("Opzioni di salvataggio")
                    .resizable(false)
                    .anchor(egui::Align2::RIGHT_TOP, egui::vec2(10.0, 45.0))
                    .show(&ctx, |ui| {
                        if ui.add(egui::TextEdit::singleline(&mut self.image_name)).changed() {
                            ctx.request_repaint();
                        }

                    egui::ComboBox::from_label("Seleziona un'estensione")
                        .selected_text(format!("{:?}", self.sel_extension))
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(self.sel_extension == FileExtension::Png, ".png").clicked() {
                                self.sel_extension = FileExtension::Png;
                                self.extension = "png".to_string();

                            }
                            if ui.selectable_label(self.sel_extension == FileExtension::Jpg, ".jpg").clicked() {
                                self.sel_extension = FileExtension::Jpg;
                                self.extension = "jpg".to_string();
                                println!("{}", self.extension);
                            }
                            if ui.selectable_label(self.sel_extension == FileExtension::Gif, ".gif").clicked() {
                                self.sel_extension = FileExtension::Gif;
                                self.extension = "gif".to_string();
                            }
                        });

            
                        if ui.button("Salva").clicked() {
                            // Salva l'immagine
                            let image = self.color_image.clone().unwrap();
                            let pix: Vec<u8> = image
                                .pixels
                                .iter()
                                .flat_map(|p| p.to_array().iter().copied().collect::<Vec<u8>>())
                                .collect();
            
                            let im: ImageBuffer<Rgba<u8>, Vec<_>> =
                                ImageBuffer::from_vec(image.width() as u32, image.height() as u32, pix)
                                    .expect("Unable to obtain ImageBuffer from vec");
            
                            let mut name: &str = &self.image_name.clone();
                            println!("{}", self.image_name);
                            let mut save_path = PathBuf::from(name);
                            save_path.set_extension(self.extension.clone());
                           
                            match self.sel_extension {
                                FileExtension::Png => {
                                    im.save_with_format(&save_path, ImageFormat::Png)
                                    .expect("Unable to save the image");
                                }
                                FileExtension::Jpg => {
                                    im.save_with_format(&save_path, ImageFormat::Jpeg)
                                    .expect("Unable to save the image");
                                }
                                FileExtension::Gif => {
                                    im.save_with_format(&save_path, ImageFormat::Gif)
                                    .expect("Unable to save the image");
                                }
                            }
                            self.is_saving_modal_open = false; // Chiudi la finestra
                        }
            
                        if ui.button("Chiudi").clicked() {
                            self.is_saving_modal_open = false; // Chiudi la finestra
                        }
                    });
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
                    match self.clipboard.clear() {
                        Ok(()) => {
                            println!("Clipboard cleared successfully");
                        }
                        Err(err) => {
                            eprintln!("Error while clearing the clipboard: {:?}", err);
                        }
                    }   
                } else {
                    frame.set_window_pos(egui::Pos2::new(screen.display_info.x as f32, screen.display_info.y as f32));
                    frame.set_fullscreen(true);
                    match self.clipboard.clear() {
                        Ok(()) => {
                            println!("Clipboard cleared successfully");
                        }
                        Err(err) => {
                            eprintln!("Error while clearing the clipboard: {:?}", err);
                        }
                    }   
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
            egui::SidePanel::left(egui::Id::new("my left panel")).exact_width(25.0).resizable(false).show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    if ui.add(egui::Button::new(egui::RichText::new("\u{270F}").size(22.0)).frame(false)).on_hover_text("Draw").clicked() {
                        self.acquiring_pen = true;
                    }
                    self.painting.ui_control(ui);
                });
            });
            egui::CentralPanel::default().show(ctx, |ui| {
                let image_size = egui::vec2(self.img.clone().unwrap().size().unwrap().x / self.window_scale, self.img.clone().unwrap().size().unwrap().y / self.window_scale);        
                let rect = get_centre_rect(ui,image_size);
                self.img.clone().unwrap().paint_at(ui,rect);
                if self.acquiring_pen {
                        self.painting.ui_content(ui,ctx,self.img.clone().unwrap().shrink_to_fit().max_size(image_size));

                }
            });
        }
        
    }

    
}

fn get_centre_rect(ui: &egui::Ui,image_size: egui::Vec2) -> egui::Rect{
    let ratio = image_size.x/image_size.y;
                
                let mut w = ui.available_width();
                if w > image_size.x {
                    w = image_size.x;
                }
                let mut h = w / ratio;
                if h > ui.available_height() {
                    h = ui.available_height();
                    w = h * ratio;
                }

                let mut rect = ui.available_rect_before_wrap();
                //println!("{} {} {} {}",rect.min.x,rect.min.y,rect.max.x,rect.max.y);
                if rect.width() > w {
                    rect.min.x += (rect.width() - w) / 2.0;
                    rect.max.x = rect.min.x + w;
                }  
                if rect.height() > h {
                    rect.min.y += (rect.height() - h) / 2.0;
                    rect.max.y = rect.min.y + h;
                }
                return rect;
}




fn take_screen(start: Pos2, max_width: usize, max_height: usize, screen_src: usize) -> Result<ColorImage, anyhow::Error> {
    let screens = Screen::all().unwrap();
    let screen = screens[screen_src - 1];

    match screen.capture() {
        Ok(image) => {

            let rgba_pixels = image.clone().into_raw();
            // Crea una ColorImage dall'immagine catturata
            let color_image = ColorImage::from_rgba_premultiplied(
                [image.width() as usize, image.height() as usize],
                &rgba_pixels,
            );

            // Definisci la regione da estrarre (usa l'intera immagine se non è necessario ridimensionare)
            let region_to_extract = Rect::from_min_max([start.x, start.x].into(), [max_width as f32, max_height as f32].into());

            // Ridimensiona l'immagine se necessario
            let resized_image = if image.width() > max_width as u32 || image.height() > max_height as u32 {
                color_image.region(&region_to_extract, Some(1.0))
            } else {
                color_image
            };

            // Stampa le dimensioni dell'immagine originale e ridimensionata per il debug
            println!("Original image width: {}", image.width());
            println!("Original image height: {}", image.height());
            println!("Resized image width: {}", resized_image.size[0]);
            println!("Resized image height: {}", resized_image.size[1]);

            // Calcola e stampa il numero di pixel
            let original_pixels = image.width() * image.height();
            let resized_pixels = resized_image.size[0] as usize * resized_image.size[1] as usize;
            println!("Original image pixels: {}", original_pixels);
            println!("Resized image pixels: {}", resized_pixels);

            Ok(resized_image)
        }
        Err(e) => {
            // Stampa l'errore in caso di problemi durante la cattura dell'immagine
            println!("Error capturing the screen: {:?}", e);
            Err(e)
        }
    }
}


/* 

fn take_screen(screen_src: usize) -> Result<ColorImage, anyhow::Error> {
    let screens = Screen::all().unwrap();
    let screen = screens[screen_src - 1];

    match screen.capture() {
    

            Ok(image) => Ok(ColorImage::from_rgba_unmultiplied(
                [700 as usize, image.height() as usize],
                image.rgba(),
            )),
            Err(e) => Err(e),
        }
    }


*/