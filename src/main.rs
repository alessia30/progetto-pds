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
            //egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(MyApp::new(cc))
            })
    )
}
pub struct Painting {
    /// in 0-1 normalized coordinates
    lines: Vec<(Vec<egui::Pos2>,Stroke)>,
    stroke: Stroke,
    temp_lines: Vec<(Vec<egui::Pos2>,Stroke)>,
}

impl Default for Painting {
    fn default() -> Self {
        Self {
            lines: Default::default(),
            temp_lines: Default::default(),
            stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
        }
    }
}

pub fn my_stroke_ui(ui: &mut crate::egui::Ui, stroke: &mut egui::epaint::Stroke, text: &str) {
    let egui::epaint::Stroke { width, color } = stroke;
    ui.vertical(|ui| {
        ui.add_space(10.0);
        ui.add(egui::DragValue::new(width).speed(0.1).clamp_range(0.0..=5.0))
            .on_hover_text("Width");
        ui.add_space(10.0);
        ui.color_edit_button_srgba(color);
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
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(5.0);
            if ui.button("Clear").clicked() {
                self.lines.clear();
                self.temp_lines.clear();
            }
            ui.add_space(10.0);
            if self.lines.is_empty() {
                self.lines.push((vec![], self.stroke.clone()));
            }
            if ui.add_enabled(self.lines.len()>1, egui::Button::new(egui::RichText::new("↩").size(18.0))).on_hover_text("Annulla").clicked() {
                println!(" lines: {}", self.lines.len());
                let _ =self.lines.pop();
                if let Some(line) = self.lines.pop() {
                    self.temp_lines.push(line);
                    self.lines.push((vec![], self.stroke.clone()));
                }
            }
            ui.add_space(10.0);
            if ui.add_enabled(self.temp_lines.len() > 0, egui::Button::new(egui::RichText::new("↪").size(18.0))).on_hover_text("Ripristina").clicked() {
                self.lines.pop();
                self.lines.push(self.temp_lines.pop().unwrap());
                self.lines.push((vec![], self.stroke.clone()));
            }

        
    }

    pub fn ui_content(&mut self, ui: &mut egui::Ui, ctx: &egui::Context,rect: egui::Rect)-> egui::Response {
        
        let (mut response, painter) = ui.allocate_painter(ui.min_size(), egui::Sense::drag());
        let to_screen = egui::emath::RectTransform::from_to(
            Rect::from_min_size(egui::Pos2::ZERO, rect.square_proportions()),
            rect,
        );
        let from_screen = to_screen.inverse();

        let current_line = self.lines.last_mut().unwrap();

        if let Some(pointer_pos) = response.interact_pointer_pos() {
            let canvas_pos = from_screen * pointer_pos;
            if current_line.0.last() != Some(&canvas_pos) {
                current_line.0.push(canvas_pos);
                current_line.1 = self.stroke.clone();
                response.mark_changed();
            }
        } else if !current_line.0.is_empty() { 
            self.lines.push((vec![], self.stroke.clone()));
            self.temp_lines.clear();
            response.mark_changed();
        }

        // Disegna le linee
        let shapes = self
                .lines
                .iter()
                .filter(|(line, _)| line.len() >= 2)
                .map(|(line, stroke)| {
                    let points: Vec<egui::Pos2> = line.iter().map(|p| to_screen * *p).collect();
                    egui::Shape::line(points, *stroke)
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
    painting:Painting,
    acquiring_pen:bool,
    screenshot: Option<egui::ColorImage>,
    cutting: bool
}

impl MyApp<'_>{
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let new_style = egui::style::WidgetVisuals {
            weak_bg_fill: egui::Color32::from_rgb(0x29, 0x29, 0x29),
            bg_fill: egui::Color32::from_rgb(0x29, 0x29, 0x29),
            bg_stroke: Stroke { width: 1., color: egui::Color32::from_rgb(0x29, 0x29, 0x29) },
            rounding: egui::Rounding { nw: 2., ne: 2., sw: 2., se: 2. },
            fg_stroke: Stroke{ width: 1., color: egui::Color32::WHITE} ,
            expansion: 4.,
        };
        cc.egui_ctx.set_visuals(egui::style::Visuals { widgets: egui::style::Widgets { 
                noninteractive: new_style, inactive: new_style, hovered: new_style, active: new_style, open: new_style
            }, ..Default::default()});

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
            painting:Painting::default(),acquiring_pen:false,screenshot:None,cutting:false
        }
    }

    fn top_panel(&mut self, ctx: &egui::Context,frame: &mut eframe::Frame){
        egui::TopBottomPanel::top("my_panel").exact_height(40.0).show(ctx, |ui| {        
            ui.horizontal_centered(|ui| {
                if ui.add(egui::Button::new(egui::RichText::new("\u{2795} Nuovo").size(14.0))).on_hover_text("Nuova Cattura").clicked() {
                    self.acquiring = true;
                    self.acquiring_pen = false;
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
                    
                    if ui.add(egui::Button::new(egui::RichText::new("\u{2699}").size(22.0)).frame(false)).on_hover_text("Modifica shortcut").clicked() {
                        self.is_shortcut_modal_open = true;
                    }
                    if self.acquired{
                        if ui.button(egui::RichText::new("salva").size(14.0)).on_hover_text("").clicked() {
                            frame.request_screenshot();                            
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
            self.counter += 1;
            //print!("{}",self.counter);            
            ctx.request_repaint();
            if self.counter == 20{
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
                        self.painting.lines.clear();
                        self.painting.temp_lines.clear();
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
            egui::SidePanel::left(egui::Id::new("my left panel")).exact_width(35.0).resizable(false).show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    if ui.add(egui::Button::new(egui::RichText::new("\u{270F}").size(22.0)).frame(false)).on_hover_text("Draw").clicked() {
                        self.acquiring_pen = true;
                    }
                    self.painting.ui_control(ui);
                });
            });
            egui::CentralPanel::default().show(ctx, |ui| {
                println!("{} {}  {} {}",ui.available_rect_before_wrap().min.x,ui.available_rect_before_wrap().min.y,ui.available_rect_before_wrap().max.x,ui.available_rect_before_wrap().max.y);
                let image_size = egui::vec2(self.img.clone().unwrap().size().unwrap().x / self.window_scale, self.img.clone().unwrap().size().unwrap().y / self.window_scale);        
                let rect = get_centre_rect(ui,image_size);
                self.img.clone().unwrap().paint_at(ui,rect);
                if self.acquiring_pen {
                    self.painting.ui_content(ui,ctx,rect);
                }
                if self.cutting {
                    let pixels_per_point = frame.info().native_pixels_per_point;
                    let region = egui::Rect::from_two_pos(
                        //egui::Pos2 { x: ui.available_rect_before_wrap().min.x + rect.min.x, y: ui.available_rect_before_wrap().min.y + rect.min.y },
                        //egui::Pos2 { x: ui.available_rect_before_wrap().max.x + rect.max.x, y: ui.available_rect_before_wrap().max.y + rect.max.y },
                        egui::Pos2 { x: rect.min.x , y: rect.min.y },
                        egui::Pos2 { x: rect.max.x , y: rect.max.y },
                    );
                    let top_left_corner = self.screenshot.clone().unwrap().region(&region, pixels_per_point);
                    image::save_buffer(
                        "top_left.png",
                        top_left_corner.as_raw(),
                        top_left_corner.width() as u32,
                        top_left_corner.height() as u32,
                        image::ColorType::Rgba8,
                    ).unwrap();
                    self.cutting=false;
                }
            });
        }
        
    }

    fn post_rendering(&mut self, _window_size: [u32; 2], frame: &eframe::Frame) {
        if let Some(screenshot) = frame.screenshot() {
            self.cutting=true;            
            self.screenshot = Some(screenshot);
        }
    }

}

fn get_centre_rect(ui: &egui::Ui,image_size: egui::Vec2) -> egui::Rect {
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

