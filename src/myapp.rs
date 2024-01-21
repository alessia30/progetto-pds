use crate::painting::Painting;
use crate::cutting::Status;
use eframe::egui;
use egui::{ Key, Modifiers, ModifierNames, KeyboardShortcut, PointerButton, Vec2, Rect, Rounding, Color32, Stroke, LayerId, Id, CursorIcon, pos2};
use screenshots::Screen;
use std::path:: PathBuf ;
use std::time::{Duration, SystemTime};
use std::thread::sleep;
use image::EncodableLayout;
use native_dialog::FileDialog;
use chrono::{ Local, DateTime };
use image::{ ImageBuffer, Rgba, ImageFormat };
use arboard::Clipboard;
use std::borrow::Cow;


static DELAYS_VALUES:[u64;4]=[0,3,5,10];

pub struct MyApp<'a>{
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
    pub start_pos:Option<egui::Pos2>,
    current_pos:Option<egui::Pos2>,
    screen:usize,
    painting:Painting,
    screenshot: Option<egui::ColorImage>,
    cutting: bool,
    pub end_pos:Option<egui::Pos2>,
    pub center_pos:Option<egui::Pos2>,
    is_cropping: bool,
    cropped: bool,
    pub selected_area:Option<Rect>,
    pub centered_area:Option<Rect>,
    pub status: Status,
    image_size: Vec2,
    file_path:PathBuf,
    timestamp:DateTime<Local>,
    image_name:String,
    clipboard:Clipboard,
    copy:bool,
    save:bool,
    default_location:PathBuf,
}

impl MyApp<'_>{
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        
        let new_style = egui::style::WidgetVisuals {
            weak_bg_fill: egui::Color32::from_rgb(0x29, 0x29, 0x29),
            bg_fill: egui::Color32::from_rgb(0x29, 0x29, 0x29),
            bg_stroke: Stroke { width: 1., color: egui::Color32::from_rgb(0x29, 0x29, 0x29) },
            rounding: egui::Rounding { nw: 2., ne: 2., sw: 2., se: 2. },
            fg_stroke: Stroke{ width: 1., color: egui::Color32::WHITE} ,
            expansion: 4.,
        };
        let hovered_style = egui::style::WidgetVisuals {
            weak_bg_fill: Color32::from_gray(70),
            bg_fill: Color32::from_gray(70),
            bg_stroke: Stroke::new(1.0, Color32::from_gray(150)), // e.g. hover over window edge or button
            fg_stroke: Stroke::new(1.5, Color32::from_gray(240)),
            rounding: Rounding::same(3.0),
            expansion: 4.
        };
        cc.egui_ctx.set_visuals(egui::style::Visuals { widgets: egui::style::Widgets { 
                noninteractive: new_style, inactive: new_style, hovered: hovered_style, active: new_style, open: new_style
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
            captures:vec!["Rettangolo", "Schermo intero"],
            delays:vec!["Nessun ritardo", "3 secondi", "5 secondi","10 secondi"],

            capture:0,delay:0,is_mac:match cc.egui_ctx.os(){ egui::os::OperatingSystem::Mac => true, _ => false },is_shortcut_modal_open:false,start_pos:None,current_pos:None,screen:1,
            painting:Painting::default(),screenshot:None,cutting:false,end_pos:None,center_pos:None, is_cropping:false,cropped:false,selected_area:None,centered_area:None,
            status:Status::None,image_size:egui::vec2(0.0, 0.0),
            file_path:PathBuf::default(),
            timestamp:Local::now(),
            image_name:String::from("Nome Immagine"),
            clipboard:Clipboard::new().expect("Unable to create clipboard"),
            copy:false,save:false,
            default_location:PathBuf::from("~"),
        }
    }

    fn top_panel(&mut self, ctx: &egui::Context,frame: &mut eframe::Frame){
        egui::TopBottomPanel::top("my_panel").exact_height(40.0).show(ctx, |ui| {        
            ui.horizontal_centered(|ui| {
                if ui.add(egui::Button::new(egui::RichText::new("\u{2795} Nuovo").size(14.0))).on_hover_text("Nuova Cattura").clicked() {
                    self.acquiring = true;
                    self.is_cropping=false;
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
                    
                    ui.visuals_mut().widgets.inactive.bg_stroke = egui::Stroke::NONE;
                    ui.visuals_mut().widgets.inactive.expansion = 1.;
                    // Bottone "Modifica Shortcut"
                    if ui.add_enabled(!self.is_cropping,egui::Button::new(egui::RichText::new("\u{2699}").size(22.0)).fill(egui::Color32::TRANSPARENT)).on_hover_text("Impostazioni").clicked() {
                        self.is_shortcut_modal_open = true;
                    }
                    if self.acquired{
                        if self.counter==1{
                            frame.request_screenshot();
                            self.is_shortcut_modal_open=false;
                            self.counter=0;
                        }
                        if ui.add_enabled(!self.is_cropping,egui::Button::image(egui::Image::new(egui::include_image!("icons/save.png")).max_height(22.0)).fill(egui::Color32::TRANSPARENT)).on_hover_text("Salva").clicked() {
                            frame.set_maximized(true);
                            self.counter=1;   
                            self.save = true;
                            self.timestamp = Local::now();
                            self.image_name = self.timestamp.format("Immagine %Y-%m-%d %H%M%S").to_string(); 
                        }
                        if ui.add_enabled(!self.is_cropping,egui::Button::image(egui::Image::new(egui::include_image!("icons/clipboard.png")).max_height(22.0)).fill(egui::Color32::TRANSPARENT)).on_hover_text("Copia").clicked() {
                            frame.set_maximized(true);
                            self.counter=1;
                            self.copy = true;
                        }
                    }
                });
                
            });
    
            // --- Gestione della window per la modifica della shortcut ---
            if self.is_shortcut_modal_open {
                egui::Window::new("Impostazioni").collapsible(false).resizable(false).anchor(egui::Align2::RIGHT_TOP, egui::vec2(10.0, 45.0)).show(&ctx, |ui| {                     
                    // Qui puoi rilevare gli eventi di input e aggiornare la variabile key
                    //ui.visuals_mut().panel_fill = Color32::from_gray(70);
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
                    
                    ui.add_space(5.0);
                    ui.label(&format!("Locazione di default: {}", self.default_location.display()));
                    ui.add_space(5.0);
                    if ui.button("Modifica locazione di default").clicked() {
                        if let Some(path) = FileDialog::new()
                            .set_location(&self.default_location)
                            .show_open_single_dir()
                            .expect("Unable to visualize the file selection window") {
                                self.default_location = Some(path).unwrap();
                            }
                    }  
                    ui.add_space(20.0);
                    ui.label(&format!("Premi dei tasti per modificare la shortcut."));
                    ui.label(&format!("Shortcut: {:?}", self.new_shortcut.format(&ModifierNames {
                        is_short: false,
                        alt: "Alt",
                        ctrl: "Ctrl",
                        shift: "Shift",
                        mac_cmd: "Cmd",
                        mac_alt: "Option",
                        concat: "+",
                    }, self.is_mac)));
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Salva shortcut").clicked() {
                            // Salva le modifiche
                            self.my_shortcut.modifiers = self.new_shortcut.modifiers;
                            self.my_shortcut.key = self.new_shortcut.key;
                            self.is_shortcut_modal_open = false; // Chiudi la finestra
                        }
                        ui.add_space(50.0);
                        if ui.button("Chiudi").clicked() {
                            self.is_shortcut_modal_open = false; // Chiudi la finestra
                        }  
                    });  
                    ui.add_space(5.0); 
                });
            }
            if !self.is_shortcut_modal_open {
                // Se la shortcut viene premuta... -> da sostituire con l'azione di cattura
                if ctx.input(|i| i.clone().consume_shortcut(&self.my_shortcut)) {
                // Esegui azioni basate sulla copia di InputState
                    self.acquiring=true;
                    self.is_cropping=false;
                    frame.set_visible(false);
                    println!("Shortcut premuta!");
                }   
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
                    self.painting.set_false(); 
                } else {
                    frame.set_window_pos(egui::Pos2::new(screen.display_info.x as f32, screen.display_info.y as f32));
                }
                match self.clipboard.clear() {
                    Ok(()) => {
                        println!("Clipboard cleared successfully");
                    }
                    Err(err) => {
                        eprintln!("Error while clearing the clipboard: {:?}", err);
                    }
                }  
            }
            if self.counter>20{  
                frame.set_fullscreen(true);
                egui::Window::new("")
                    .title_bar(false)
                    .frame(egui::Frame{fill:egui::Color32::from_white_alpha(10), ..Default::default()})
                    .movable(false)
                    .fixed_pos(frame.info().window_info.position.unwrap())
                    .show(ctx, |ui| {
                    
                    self.img.clone().unwrap().paint_at(ui, ctx.screen_rect());                  

                    let (response, painter) = ui.allocate_painter(ctx.screen_rect().size(),egui::Sense::click_and_drag() );            
                    painter.rect_filled( ctx.screen_rect(), Rounding::ZERO, Color32::from_rgba_premultiplied(0, 0, 0, 130));
                    ctx.set_cursor_icon(egui::CursorIcon::Crosshair);
                    if self.acquired{ 
                        let rect = Rect::from_two_pos(self.start_pos.unwrap(), self.current_pos.unwrap());
                        let pixels_per_point = frame.info().native_pixels_per_point;
                        self.color_image=Some(self.color_image.clone().unwrap().region(&rect, pixels_per_point));
                        self.handle= Some(ctx.load_texture("handle", self.color_image.clone().unwrap(),egui::TextureOptions::LINEAR));
                        let sized_image = egui::load::SizedTexture::new(self.handle.clone().unwrap().id(), egui::vec2(self.color_image.clone().unwrap().size[0] as f32, self.color_image.clone().unwrap().size[1] as f32));
                        self.img = Some(egui::Image::from_texture(sized_image));
                        self.acquiring=false;
                        self.painting.set_false();
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
                        if self.start_pos == self.current_pos{
                            return;
                        }
                        self.acquired=true;
                    }
                });
            }
        }

        if self.acquired && !self.acquiring{
            self.top_panel(ctx, frame);
            egui::SidePanel::left(egui::Id::new("my left panel")).exact_width(55.0).resizable(false).show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    if ui.add(egui::Button::image(egui::Image::new(egui::include_image!("icons/cut.png")).max_height(19.0))).on_hover_text("Ritaglia").clicked() {
                        self.is_cropping=true;
                        self.is_shortcut_modal_open=false;
                        println!("taglia");
                    }
                    ui.add_space(10.0);
                    if self.is_cropping {
                        let mut pressed = false;
                        ui.with_layer_id(
                            LayerId::new(egui::Order::Foreground, Id::from("Save")),
                            |ui| {
                                    let save = ui.add(egui::Button::image(egui::Image::new(egui::include_image!("icons/conferma.png")).max_height(22.0)).fill(egui::Color32::TRANSPARENT));
                                    ui.add_space(10.0);
                                    let cancel = ui.add(egui::Button::image(egui::Image::new(egui::include_image!("icons/annulla.png")).max_height(22.0)).fill(egui::Color32::TRANSPARENT));
                                    ui.add_space(10.0);

                                    let save_rect = save.rect;
                                    let cancel_rect = cancel.rect;
                                    
                                    let pointer_pos = ctx.pointer_hover_pos();
                                    
                                    if pointer_pos.is_some() {
                                        if save_rect.contains(pointer_pos.unwrap()) {
                                            ctx.set_cursor_icon(CursorIcon::PointingHand);
                                            save.highlight();
                                            
                                            if ctx.input(|i| i.pointer.primary_clicked()) {
                                                if self.get_selected_area().is_some() {
                                
                                                    let pixels_per_point = frame.info().native_pixels_per_point;
                                                    let region = self.get_selected_area().unwrap();
                                                    let im = self.get_centered_area().unwrap();
                                                    println!("selected area {:?}",region);
                                                    println!("centered area: {:?}", self.get_centered_area());
                                                                
                                                    let scale_x = im.width()/region.width();
                                                    let scale_y = im.height()/region.height();
                                                    let rw = (self.color_image.as_ref().unwrap().width() as f32 / self.window_scale ) / scale_x ;
                                                    let rh = (self.color_image.as_ref().unwrap().height() as f32 / self.window_scale ) / scale_y ;

                                                    let rect = Rect::from_min_size(pos2((region.min.x-im.min.x)*rw/region.width(), (region.min.y-im.min.y)*rh/region.height()), Vec2::new(rw, rh));
        
                                                    self.color_image=Some(self.color_image.clone().unwrap().region(&rect, pixels_per_point));
                                                    self.handle= Some(ctx.load_texture("handle", self.color_image.clone().unwrap(),egui::TextureOptions::LINEAR));                                            
                                                    let sized_image = egui::load::SizedTexture::new(self.handle.clone().unwrap().id(), egui::vec2(self.color_image.clone().unwrap().size[0] as f32, self.color_image.clone().unwrap().size[1] as f32));
                                                    self.img = Some(egui::Image::from_texture(sized_image));
                                                    self.window_scale=ctx.pixels_per_point();                 
                                                    self.painting.prec_area=Some(im);
                                                    self.painting.cutted_area=Some(region);
                                                }
                                                
                                                //save in memory in case of cancel
                                                ctx.memory_mut(|mem| {
                                                    mem.data.insert_temp(Id::from("Prev_area"), self.get_selected_area());
                                                });
                                               
                                                pressed = true; 
                                            }
                                        }
                                        else if cancel_rect.contains(pointer_pos.unwrap()) {
                                            ctx.set_cursor_icon(CursorIcon::PointingHand);
                                            cancel.highlight();
                                            
                                            if ctx.input(|i| i.pointer.primary_clicked()) {
                                                let prev_area = ctx.memory_mut(|mem| {
                                                    mem.data.get_temp::<Option<Rect>>(Id::from("Prev_area"))
                                                }).unwrap_or_else(|| {None});
                                                
                                                self.set_selected_area(prev_area);
                                                pressed = true;
                                            }  
                                        }                                      
                                    }
                                }                                
                            );
                            
                            if pressed {
                                ctx.request_repaint();
                                self.is_cropping = false; 
                                self.cropped = false;
                            }
                            
                        }
                        
                    if !self.is_cropping {
                        self.painting.ui_control(ui);
                    }
                        
                });
            });
            egui::CentralPanel::default().show(ctx, |ui| {

                self.image_size = egui::vec2(self.img.clone().unwrap().size().unwrap().x / self.window_scale, self.img.clone().unwrap().size().unwrap().y / self.window_scale);        
                let rect = get_centre_rect(ui,self.image_size);
                self.set_selected_area(Some(rect));
                self.set_centered_area(Some(rect));
                self.img.clone().unwrap().paint_at(ui,rect);
                self.painting.ui_content(ui,rect);
                // println!("centered area: {:?}", self.get_centered_area());

                if self.cutting {
                    let pixels_per_point = frame.info().native_pixels_per_point;
                    let top_left_corner = self.screenshot.clone().unwrap().region(&rect, pixels_per_point);
                    println!("top left corner: {} {:?}",top_left_corner.pixels.len(),top_left_corner.size);
                    frame.set_maximized(false);
                    if self.copy {
                        if let Err(e) = self.clipboard.set_image(arboard::ImageData {width: top_left_corner.width(), height: top_left_corner.height(), bytes: Cow::from(top_left_corner.as_raw())}) {
                            println!("Unable to copy in the clipboard: {:?}", e);
                        }
                        self.copy = false;
                    }
                    if self.save {
                        if let Some(path) = FileDialog::new()
                            .set_location(&self.default_location)
                            .add_filter("PNG", &["png"])
                            .add_filter("JPG", &["jpg"])
                            .add_filter("GIF", &["gif"])
                            .set_filename(self.image_name.as_str())
                            .show_save_single_file()
                            .expect("Unable to visualize the file selection window") {
                                self.file_path = Some(path).unwrap();
                            }
                        
                        let im: ImageBuffer<Rgba<u8>, Vec<_>> = ImageBuffer::from_vec(
                            top_left_corner.width() as u32,
                            top_left_corner.height() as u32,
                            top_left_corner.as_raw().to_vec(),
                        )
                        .expect("Unable to obtain ImageBuffer from vec");

                        match self.file_path.extension().and_then(|ext| ext.to_str()) {
                            Some("png") => {
                                im.save_with_format(&self.file_path, ImageFormat::Png)
                                    .expect("Unable to save the image");
                            }
                            Some("jpg") => {
                                im.save_with_format(&self.file_path, ImageFormat::Jpeg)
                                    .expect("Unable to save the image");
                            }
                            Some("gif") => {
                                im.save_with_format(&self.file_path, ImageFormat::Gif)
                                    .expect("Unable to save the image");
                            }
                            _ => {
                                println!("Formato non supportato");
                            }
                        }
                        self.save = false;
                    }
                    self.cutting=false;
                }
                if self.is_cropping {
                    egui::Window::new("")
                    .title_bar(false)
                    .frame(egui::Frame{fill:egui::Color32::from_white_alpha(10), ..Default::default()})
                    .movable(false)
                    .fixed_rect(rect)
                    .constraint_to(Rect::from_min_max(rect.min, rect.max))
                    .show(ctx, |ui| {   

                        let (response, mut painter) = ui.allocate_painter(self.image_size,egui::Sense::click_and_drag() );                    
                        painter.rect_filled(ctx.screen_rect(), Rounding::ZERO, Color32::from_rgba_premultiplied(0, 0, 0, 130));
                                        
                        if !self.cropped {
                            let image_rect=self.get_centered_area().unwrap();
                            let pointer_pos = ctx.pointer_hover_pos();
                            if pointer_pos.is_some() {
                                if rect.contains(pointer_pos.unwrap()) {
                                    ctx.set_cursor_icon(egui::CursorIcon::Crosshair);
                                }
                            }

                            if response.drag_started_by(PointerButton::Primary){
                                self.start_pos = response.interact_pointer_pos();
                                println!("START");
                            }
                            if response.dragged_by(PointerButton::Primary){
                                println!("DRAG");
                                let mut pos = response.interact_pointer_pos().unwrap();
                                check_pos( &mut pos,image_rect);
                                self.current_pos=Some(pos);
                                painter.rect(Rect::from_two_pos(self.start_pos.unwrap(), self.current_pos.unwrap()), Rounding::ZERO,  Color32::from_rgba_premultiplied(30, 30, 30, 30),Stroke::new(2.0, Color32::WHITE) );
                                
                            }
                            if response.drag_released_by(PointerButton::Primary){  
                                let mut pos = response.interact_pointer_pos().unwrap();
                                check_pos( &mut pos,image_rect);
                                self.end_pos=Some(pos);
                                ctx.set_cursor_icon(egui::CursorIcon::Default);
                                self.cropped=true;
                            }     
                        }
                        if self.cropped { 
                            self.set_selected_area(Some(Rect::from_two_pos(self.start_pos.unwrap(), self.end_pos.unwrap())));
                            //print!("new selection: {:?}", self.selected_area);
                            painter.rect(self.selected_area.unwrap(), Rounding::ZERO,  Color32::from_rgba_premultiplied(30, 30, 30, 30),Stroke::new(2.0, Color32::WHITE) );
                            self.scale_selection(ctx, frame, &mut painter);         
                        }
                    });
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

fn get_centre_rect(ui: &egui::Ui,image_size: Vec2) -> egui::Rect {
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

fn check_pos(pos:  &mut egui::Pos2,image_rect: egui::Rect){
    if pos.x < image_rect.min.x {
        pos.x = image_rect.min.x;
    } 
    if pos.y > image_rect.max.y {
        pos.y = image_rect.max.y; 
    }
    if pos.x > image_rect.max.x {
        pos.x = image_rect.max.x;
    } 
    if pos.y < image_rect.min.y {
        pos.y = image_rect.min.y; 
    }
}