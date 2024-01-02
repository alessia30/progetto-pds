#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{egui, App, WindowInfo};
use eframe::egui_glow::painter;
use egui_extras::syntax_highlighting::highlight;
use image::imageops::crop_imm;
use std::sync::mpsc;
use std::thread::{JoinHandle, Thread};
use screenshots::Screen;
use std::time::{Duration, SystemTime};
use std::thread::sleep;
use image::EncodableLayout;
use egui_extras::RetainedImage;
use egui::{ Key, Modifiers, ModifierNames, KeyboardShortcut, Order, PointerButton, RawInput, Vec2, Widget, Rect, Rounding, Color32, Stroke, Pos2, LayerId, CursorIcon, Id, Layout, Button, Context, pos2, Painter, CentralPanel, ColorImage, Shape };

static DELAYS_VALUES:[u64;4]=[0,3,5,10];

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Status {
    None,
    Select,
    TopLeft, 
    TopMid,
    TopRight,
    MidLeft,
    MidRight,
    BotLeft,
    BotMid,
    BotRight,
    Move,
}


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
    end_pos: Option<egui::Pos2>,
    drag_pos: Pos2,
    status: Status,
    is_cropping: bool,
    cropped: bool,
    image_sz: Vec2,
    selected_area: Option<Rect>,
    temp_image:Option<ColorImage>,
}

impl MyApp<'_>{
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
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
            capture:0,delay:0,is_mac:true,is_shortcut_modal_open:false,start_pos:None,current_pos:None, end_pos:None, drag_pos: pos2(0.0, 0.0),
            status: Status::None, is_cropping:false, cropped: false, image_sz: egui::vec2(0.0, 0.0), selected_area: None,
            temp_image:None,
        }
    }

    fn get_selected_area (&self) -> Option<Rect> {
        self.selected_area
    }

    fn get_status (&self) -> Status {
        self.status
    }

    pub fn get_temp_image(&self) -> Option<ColorImage> {
        self.temp_image.clone()
    }

    fn set_selected_area(&mut self, new_area: Option<Rect>) {
        self.selected_area = new_area;
    }
    
    fn set_status(&mut self, new_status: Status) {
        self.status = new_status;
    }
    
    pub fn set_temp_image(&mut self, new_image: Option<ColorImage>) {
        self.color_image = new_image.clone();
        self.temp_image = new_image.clone();
    }

    fn get_position(&mut self, ctx: &Context, rect: Rect) -> Option<Pos2> {
        let pointer = ctx.input(|i| i.pointer.interact_pos())?;
        let side_grab_radius = ctx.style().interaction.resize_grab_radius_side;
        let corner_grab_radius = ctx.style().interaction.resize_grab_radius_corner;
                            
        if (rect.left() - pointer.x).abs() <= side_grab_radius {self.status=Status::MidLeft;}
        if (rect.right() - pointer.x).abs() <= side_grab_radius {self.status=Status::MidRight;}
        if (rect.top() - pointer.y) <= side_grab_radius {self.status=Status::TopMid;}
        if (rect.bottom() - pointer.y) <= side_grab_radius {self.status=Status::BotMid;}

        if rect.right_bottom().distance(pointer) < corner_grab_radius {self.status=Status::BotRight}
        if rect.right_top().distance(pointer) < corner_grab_radius {self.status=Status::TopRight;}
        if rect.left_top().distance(pointer) < corner_grab_radius {self.status=Status::TopLeft;}
        if rect.left_bottom().distance(pointer) < corner_grab_radius {self.status=Status::BotLeft;}
        return Some(pointer);
                            
    }

    fn crop_screen (&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
            let window_size = _frame.info().window_info.size;
            let mut painter = ctx.layer_painter(LayerId::background());
            let image = RetainedImage::from_color_image(
                "Preview Image",
                self.get_temp_image().expect("Image must be defined"),
            );

            // painter.set_clip_rect(Rect::from_min_size(pos2(0.0, 0.0), window_size));
            painter.image(
                image.texture_id(ctx),
                Rect::from_min_size(pos2(0.0, 0.0), self.image_sz),
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );

            self.show_selected_area(ctx, _frame, &mut painter);

            if ctx.pointer_hover_pos().is_some()
            //         && !(save_rect.contains(ctx.pointer_hover_pos().unwrap())
            //             || cancel_rect.contains(ctx.pointer_hover_pos().unwrap()))
                {
                    self.select_area(ctx, _frame);
                }


        
    }

    fn select_area (&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        println!("select_area");
        if ctx.input(|i| i.pointer.primary_released())
            && self.get_status() == Status::Select
        {
            self.set_status(Status::None);
        }
        //Otherwhise start the selection, if it is not started and a mouse pression is detected, or continue it until the mouse is released
        else if ctx.input(|i| i.pointer.primary_down())
            && (self.get_status() == Status::Select
                || self.get_status() == Status::None)
        {
            self.set_status(Status::Select);

            //Get drag initial and actual position
            let mut init_pos = ctx.input(|i| {
                i.pointer
                    .press_origin()
                    .expect("Press origin must be defined")
            });
            let mut drag_pos = ctx
                .pointer_hover_pos()
                .expect("Hover position must be some");

            //Update the saved area
            if init_pos != drag_pos {
                (init_pos, drag_pos) =
                    self.check_coordinates(init_pos, drag_pos, _frame.info().window_info.size);
                    println!("check coord: {:?}", self.check_coordinates(init_pos, drag_pos, _frame.info().window_info.size));
                self.set_selected_area(Some(Rect::from_min_max(init_pos, drag_pos)));
            }
        }
    }

    fn show_selected_area(&mut self, ctx: &Context, _frame: &mut eframe::Frame, painter: &mut Painter) {
        let window_size = _frame.info().window_info.size;

        match self.get_selected_area() {
            Some(sel) => {
                println!("selected area: {:?}", sel);
                        let min = sel.min;
                        let max = sel.max;
        
                        // let min_x = min.x;
                        // let max_x = max.x;
                        // let min_y = min.y;
                        // let max_y = max.y;
        
                        //Draw the overlay only on the screen parts that are not selected. Achieved using 4 rectangles
                        painter.rect_filled(
                            Rect::from_min_max(min, max),
                            0.0,
                            Color32::from_white_alpha(100),
                        );

                        self.scale_selection(ctx, _frame, painter);

            }
            None => painter.rect_filled(
                Rect::from_min_size(pos2(0.0, 0.0), window_size), 
                0.0,
                Color32::from_black_alpha(100)),
        }

    }

    fn scale_selection(&mut self, ctx: &Context, _frame: &mut eframe::Frame, painter: &mut Painter) {
        let sel = self
        .get_selected_area()
        .expect("Selected area must be Some");
        let status = self.get_status();
        println!("{:?}", status);

        if status != Status::Select {
            let point_dim = Vec2::splat(10.0);

            //disegna punti
            let tl_point = Rect::from_center_size(sel.left_top(), point_dim);
            let tm_point = Rect::from_center_size(sel.center_top(), point_dim);
            let tr_point = Rect::from_center_size(sel.right_top(), point_dim);
            let ml_point = Rect::from_center_size(sel.left_center(), point_dim);
            let mr_point = Rect::from_center_size(sel.right_center(), point_dim);
            let bl_point = Rect::from_center_size(sel.left_bottom(), point_dim);
            let bm_point = Rect::from_center_size(sel.center_bottom(), point_dim);
            let br_point = Rect::from_center_size(sel.right_bottom(), point_dim);


            //i punti sono disegnati solo mentre non sto muovendo la selezione
            if status != Status::Move {
                painter.set_layer_id(LayerId::new(Order::Middle, Id::from("points")));
                painter.rect_filled(tl_point, 0.0, Color32::from_rgb(100, 10, 10));
                painter.rect_filled(tr_point, 0.0, Color32::from_rgb(100, 10, 10));
                painter.rect_filled(bl_point, 0.0, Color32::from_rgb(100, 10, 10));
                painter.rect_filled(br_point, 0.0, Color32::from_rgb(100, 10, 10));

                painter.rect_filled(tm_point, 0.0, Color32::from_rgb(100, 10, 10));
                painter.rect_filled(ml_point, 0.0, Color32::from_rgb(100, 10, 10));
                painter.rect_filled(mr_point, 0.0, Color32::from_rgb(100, 10, 10));
                painter.rect_filled(bm_point, 0.0, Color32::from_rgb(100, 10, 10));
            }

            match ctx.pointer_hover_pos() {
                Some(pos) => {
                    let mut new_status = Status::None;

                    if tm_point.contains(pos) || status == Status::TopMid {
                        ctx.set_cursor_icon(CursorIcon::ResizeVertical);
                        new_status = Status::TopMid;
                    } else if ml_point.contains(pos) || status == Status::MidLeft {
                        ctx.set_cursor_icon(CursorIcon::ResizeHorizontal);
                        new_status = Status::MidLeft;
                    } else if mr_point.contains(pos) || status == Status::MidRight {
                        ctx.set_cursor_icon(CursorIcon::ResizeHorizontal);
                        new_status = Status::MidRight;
                    } else if bm_point.contains(pos) || status == Status::BotMid {
                        ctx.set_cursor_icon(CursorIcon::ResizeVertical);
                        new_status = Status::BotMid;
                    } else if tl_point.contains(pos) || status == Status::TopLeft {
                        ctx.set_cursor_icon(CursorIcon::ResizeNwSe);
                        new_status = Status::TopLeft;
                    } else if tr_point.contains(pos) || status == Status::TopRight {
                        ctx.set_cursor_icon(CursorIcon::ResizeNeSw);
                        new_status = Status::TopRight;
                    } else if bl_point.contains(pos) || status == Status::BotLeft {
                        ctx.set_cursor_icon(CursorIcon::ResizeNeSw);
                        new_status = Status::BotLeft;
                    } else if br_point.contains(pos) || status == Status::BotRight {
                        ctx.set_cursor_icon(CursorIcon::ResizeNwSe);
                        new_status = Status::BotRight;
                    } else if sel.contains(pos) || status == Status::Move {
                        ctx.set_cursor_icon(CursorIcon::Grab);
                        new_status = Status::Move;
                    }
                    println!("{:?}", new_status);
                    
                    if new_status != Status::None {
                        print!("update area\n");
                        self.update_area(ctx, _frame, pos, new_status);
                    }

                }

                None => ctx.set_cursor_icon(CursorIcon::Crosshair),
            }
        }
     }

    fn update_area (&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame, pos: Pos2, status: Status) {
        let sel = self.get_selected_area().expect("Selected area must be some when updating it");
        let mut new_min = sel.min;
        let mut new_max = sel.max;
        println!("pos: {:?}", pos);

        if ctx.input(|i| i.pointer.primary_down()) {
            match self.get_status() {
                Status::None => self.set_status(status), //set status if enters with None
                Status::Select => unreachable!("Sould not be un select mode during update"),
                Status::TopLeft => new_min = pos,
                Status::TopMid => {new_min = pos2(sel.min.x, pos.y); println!("{:?}", new_min)},
                Status::TopRight => {
                    new_min = pos2(sel.min.x, pos.y);
                    new_max = pos2(pos.x, sel.max.y);
                }
                Status::MidLeft => new_min = pos2(pos.x, sel.min.y),
                Status::MidRight => new_max = pos2(pos.x, sel.max.y),
                Status::BotLeft => {
                    new_min = pos2(pos.x, sel.min.y);
                    new_max = pos2(sel.max.x, pos.y);
                }
                Status::BotMid => new_max = pos2(sel.max.x, pos.y),
                Status::BotRight => new_max = pos,
                Status::Move =>  {
                    ctx.set_cursor_icon(CursorIcon::Grabbing);

                    //save distance from center of selcted area if move jsut started
                    let center_distance = 
                        match ctx.memory(|mem| mem.data.get_temp(Id::from("center_dist"))) {
                            Some(distance) => distance,
                            None => {
                                let start_coord = ctx.pointer_interact_pos()
                                .expect("Pointer position must be found")
                                .to_vec2();
                                
                                let distance = start_coord - sel.center().to_vec2();
                                ctx.memory_mut(|mem| {
                                    mem.data.insert_temp(Id::from("center_dist"), distance)
                                });
                                distance
                            }
                        };
                    println!("center distance: {:?}", center_distance); 
                    
                    //update center
                    let mut new_center = pos2(pos.x - center_distance.x, pos.y - center_distance.y);

                    //check that new center is inside window
                    {
                        let size = sel.size();
                        let window_size = _frame.info().window_info.size;

                        new_center = new_center.clamp((size/2.).to_pos2(), (window_size.to_pos2() - pos2(size[0]/2. , size[1]/2.)).to_pos2());
                        println!("new center: {:?}", new_center);
                    }

                    //update area with new center
                    self.set_selected_area(Some(Rect::from_center_size(new_center, sel.size())));
                }
                    
            }
            //update selected area if not in Move mode
            if self.get_status() != Status::Move {
                (new_min, new_max) = 
                    self.check_coordinates(new_min, new_max, _frame.info().window_info.size);
                self.set_selected_area(Some(Rect::from_min_max(new_min, new_max)));
            }
        }
        //mouse released, reset status and used values
        else {
            if self.get_status() == Status::Move {
                ctx.memory_mut(|mem| mem.data.remove::<Vec2>(Id::from("center_dist")));
            }
            self.set_status(Status::None);
        }
    }

    fn check_coordinates(&mut self, start: Pos2, end: Pos2, window_size: Vec2) -> (Pos2, Pos2) {
        let mut init_pos = start.clamp(pos2(0., 0.), window_size.to_pos2());
        let mut end_pos = end.clamp(pos2(0., 0.), window_size.to_pos2());

        let mut status = self.get_status();

        if init_pos.x > end_pos.x {
            let tmp = init_pos.x;
            init_pos.x = end_pos.x;
            end_pos.x = tmp;

            if status == Status::MidLeft {
                status = Status::MidRight;
            } else if status == Status::MidRight {
                status = Status::MidLeft;
            } else if status == Status::TopLeft {
                status = Status::TopRight;
            } else if status == Status::TopRight {
                status = Status::TopLeft;
            } else if status == Status::BotLeft {
                status = Status::BotRight;
            } else if status == Status::BotRight {
                status = Status::BotLeft;
            }
        }

        if init_pos.y > end_pos.y {
            let tmp = init_pos.y;
            init_pos.y = end_pos.y;
            end_pos.y = tmp;

            if status == Status::TopMid {
                status = Status::BotMid;
            } else if status == Status::BotMid {
                status = Status::TopMid;
            } else if status == Status::TopLeft {
                status = Status::BotLeft;
            } else if status == Status::TopRight {
                status = Status::BotRight;
            } else if status == Status::BotLeft {
                status = Status::TopLeft;
            } else if status == Status::BotRight {
                status = Status::TopRight;
            }
        }

        self.set_status(status);
        (init_pos, end_pos)

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
            println!("{}",self.counter);            
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
                        let rect = Rect::from_two_pos(self.start_pos.unwrap(), self.current_pos.unwrap());
                        let pixels_per_point = frame.info().native_pixels_per_point;
                        self.color_image=Some(self.color_image.clone().unwrap().region(&rect, pixels_per_point));
                        self.handle= Some(ctx.load_texture("handle", self.color_image.clone().unwrap(),egui::TextureOptions::LINEAR));
                        let sized_image = egui::load::SizedTexture::new(self.handle.clone().unwrap().id(), egui::vec2(self.color_image.clone().unwrap().size[0] as f32, self.color_image.clone().unwrap().size[1] as f32));
                        self.img = Some(egui::Image::from_texture(sized_image));
                        self.window_scale=self.img.clone().unwrap().size().unwrap().x/(rect.max.x-rect.min.x);
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
                        // print!("pulsante premuto");                                
                        frame.set_visible(false);
                                         
                    }
                    ui.add_space(10.0);
                    if ui.button("\u{2795} Ritaglia").clicked() {
                        self.is_cropping=true;
                        self.set_temp_image(self.color_image.clone());
                        println!("taglia");
                    }

                    if self.is_cropping {
                        ui.add_space(100.0);
                        if ui.button("Salva").clicked() {

                            let rect = self.get_selected_area().unwrap();
                            let pixels_per_point = frame.info().native_pixels_per_point;
                            self.color_image=Some(self.color_image.clone().unwrap().region(&rect, pixels_per_point));
                            self.handle= Some(ctx.load_texture("handle", self.color_image.clone().unwrap(),egui::TextureOptions::LINEAR));
                            let sized_image = egui::load::SizedTexture::new(self.handle.clone().unwrap().id(), egui::vec2(self.color_image.clone().unwrap().size[0] as f32, self.color_image.clone().unwrap().size[1] as f32));
                            self.img = Some(egui::Image::from_texture(sized_image));
                            self.window_scale=self.img.clone().unwrap().size().unwrap().x/(rect.max.x-rect.min.x);
                            self.is_cropping=false;
                            frame.set_fullscreen(false);                 
                        }
                        
                        ui.add_space(10.0);
                        if ui.button("Cancella").on_hover_text("Nuova Cattura").clicked() {
                            self.is_cropping=false;                        
                        }
                    }
                
                });
                
            });
            egui::CentralPanel::default().show(ctx, |ui| {
                self.image_sz = egui::vec2(self.img.clone().unwrap().size().unwrap().x / self.window_scale, self.img.clone().unwrap().size().unwrap().y / self.window_scale);      
                let mut pos_1 = Pos2::new(0.0, 0.0);
                let mut pos_2 = Pos2::new(0.0, 0.0);
            
                ui.centered_and_justified(|ui|{
                        let image_captured = ui.add(self.img.clone().unwrap().shrink_to_fit().max_size(self.image_sz));
                        pos_1 = image_captured.rect.left_top();
                        pos_2 = image_captured.rect.right_bottom();
                        self.set_selected_area(Some(Rect::from_two_pos(pos_1, pos_2)));
                });

                if self.is_cropping {
                    
                    egui::Window::new("")
                    .title_bar(false)
                    .frame(egui::Frame{fill:egui::Color32::from_white_alpha(10), ..Default::default()})
                    .movable(false)
                    .constraint_to(Rect::from_min_max(pos_1, pos_2))
                    .show(ctx, |ui| {
 
                    let (response, mut painter) = ui.allocate_painter(self.image_sz,egui::Sense::click_and_drag() );                    
                    println!("painter");
                    println!("selection: {:?}", self.selected_area);
                    
                    painter.rect_filled(ctx.screen_rect(), Rounding::ZERO, Color32::from_rgba_premultiplied(0, 0, 0, 130));
                    println!("painter rect");
                    
                    if !self.cropped {
                        ctx.set_cursor_icon(egui::CursorIcon::Crosshair);
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
                            self.end_pos=response.interact_pointer_pos();
                            ctx.set_cursor_icon(egui::CursorIcon::Default);
                            self.cropped=true;
                        }

                    }

                    if self.cropped { 
                        self.set_selected_area(Some(Rect::from_two_pos(self.start_pos.unwrap(), self.end_pos.unwrap())));
                        print!("new selection: {:?}", self.selected_area);
                        // let rect = Rect::from_two_pos(self.start_pos.unwrap(), self.end_pos.unwrap());
                        // let Rect {min, max } = rect;
                        let sel = self.get_selected_area().expect("Selected area must be some when updating it");
                        let mut new_min = sel.min;
                        let mut new_max = sel.max;
                        painter.rect(Rect::from_two_pos(new_min, new_max), Rounding::ZERO,  Color32::from_rgba_premultiplied(30, 30, 30, 30),Stroke::new(2.0, Color32::WHITE) );
                        self.scale_selection(ctx, frame, &mut painter);
                        print!("updated area: {:?}", self.selected_area);

                        if ctx.pointer_hover_pos().is_some()
                        // && !(save_rect.contains(ctx.pointer_hover_pos().unwrap())
                        // || cancel_rect.contains(ctx.pointer_hover_pos().unwrap()))
                        {
                            self.select_area(ctx, frame);
                        }

    
                            // let rounding = ui.visuals().window_rounding;
                            // let mut points = Vec::new();
                            // if self.status == Status::MidRight {
                            //     min = min.y + rounding.ne;
                            //     points.push(pos2(max.x, max.y - rounding.se));
                            // }

                            // if self.status == Status::TopMid {
                            //     points.push(pos2(min.x + rounding.nw, min.y));
                            //     points.push(pos2(max.x - rounding.ne, min.y));
                            // }
                            
                    
                    }


                

                });


                }
                
            });
        }
        
    }
}