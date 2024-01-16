
use crate::painting::Painting;
use eframe::egui::{self};
use eframe::epaint::vec2;
use egui::{ Key, Modifiers, ModifierNames, KeyboardShortcut, PointerButton, RawInput, Vec2, Widget, Rect, Rounding, Color32, Stroke, LayerId, Id, CursorIcon, pos2, Context, Painter, Pos2, Order };
use screenshots::Screen;
use std::time::{Duration, SystemTime};
use std::thread::sleep;
use image::EncodableLayout;

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
    start_pos:Option<egui::Pos2>,
    current_pos:Option<egui::Pos2>,
    end_pos:Option<egui::Pos2>,
    center_pos:Option<egui::Pos2>,
    screen:usize,
    painting:Painting,
    acquiring_pen:bool,
    screenshot: Option<egui::ColorImage>,
    cutting: bool,
    is_cropping: bool,
    cropped: bool,
    selected_area:Option<Rect>,
    centered_area:Option<Rect>,
    status: Status,
    image_size: Vec2,
}

impl MyApp<'_>{
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        
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
            captures:vec!["Rettangolo", "Schermo intero", "Finestra"],
            delays:vec!["Nessun ritardo", "3 secondi", "5 secondi","10 secondi"],
            capture:0,delay:0,is_mac:match cc.egui_ctx.os(){ egui::os::OperatingSystem::Mac => true, _ => false },is_shortcut_modal_open:false,start_pos:None,current_pos:None,end_pos:None,center_pos:None,screen:1,
            painting:Painting::default(),acquiring_pen:false,screenshot:None,cutting:false, is_cropping:false,cropped:false,selected_area:None,centered_area:None,status:Status::None,image_size:egui::vec2(0.0, 0.0),
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
                    
                    ui.visuals_mut().widgets.inactive.bg_stroke = egui::Stroke::NONE;
                    ui.visuals_mut().widgets.inactive.expansion = 1.;
                    // Bottone "Modifica Shortcut"
                    if ui.add(egui::Button::new(egui::RichText::new("\u{2699}").size(22.0)).fill(egui::Color32::TRANSPARENT)).on_hover_text("Modifica shortcut").clicked() {
                        self.is_shortcut_modal_open = true;
                    }
                    if self.acquired{
                        if ui.add(egui::Button::image(egui::Image::new(egui::include_image!("icons/save.png")).max_height(22.0)).fill(egui::Color32::TRANSPARENT)).on_hover_text("Salva").clicked() {
                            frame.request_screenshot();                            
                        }
                        if ui.add(egui::Button::image(egui::Image::new(egui::include_image!("icons/clipboard.png")).max_height(22.0)).fill(egui::Color32::TRANSPARENT)).on_hover_text("Copia").clicked() {
                            frame.request_screenshot();                            
                        }
                    }
                });
                
            });
    
            // --- Gestione della window per la modifica della shortcut ---
            if self.is_shortcut_modal_open {
                egui::Window::new("Modifica shortcut").collapsible(false).resizable(false).anchor(egui::Align2::RIGHT_TOP, egui::vec2(10.0, 45.0)).show(&ctx, |ui| {                     
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
                    ui.label(&format!("Hai premuto: {:?}", self.new_shortcut.format(&ModifierNames {
                        is_short: false,
                        alt: "Alt",
                        ctrl: "Ctrl",
                        shift: "Shift",
                        mac_cmd: "Cmd",
                        mac_alt: "Option",
                        concat: "+",
                    }, self.is_mac)));
                    ui.add_space(15.0);
                    ui.horizontal(|ui| {
                        ui.add_space(5.0);
                        if ui.button("Salva").clicked() {
                            // Salva le modifiche
                            self.my_shortcut.modifiers = self.new_shortcut.modifiers;
                            self.my_shortcut.key = self.new_shortcut.key;
                            self.is_shortcut_modal_open = false; // Chiudi la finestra
                        }
                        ui.add_space(30.0);
                        if ui.button("Chiudi").clicked() {
                            self.is_shortcut_modal_open = false; // Chiudi la finestra
                        }     
                        ui.add_space(5.0);
                    });
                    ui.add_space(5.0); 
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

    fn get_selected_area (&self) -> Option<Rect> {
        self.selected_area
    }
    
    fn get_status (&self) -> Status {
        self.status
    }

    fn get_centered_area (&self) -> Option<Rect> {
        self.centered_area
    }
    
    fn set_selected_area(&mut self, new_area: Option<Rect>) {
        self.selected_area = new_area;
    }

    fn set_centered_area(&mut self, new_area: Option<Rect>) {
        self.centered_area = new_area;
    }
    
    fn set_status(&mut self, new_status: Status) {
        self.status = new_status;
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
                        self.update_area(ctx, _frame, pos, new_status, painter);
                    }

                }

                None => ctx.set_cursor_icon(CursorIcon::Crosshair),
            }
        }
     }

    fn update_area (&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame, pos: Pos2, status: Status, painter: &mut Painter) {
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
                                self.center_pos=Some(sel.center());
                                let start_coord = ctx.pointer_interact_pos()
                                .expect("Pointer position must be found")
                                .to_vec2();
                                
                                let distance = start_coord - self.center_pos.unwrap().to_vec2();
                                ctx.memory_mut(|mem| {
                                    mem.data.insert_temp(Id::from("center_dist"), distance)
                                });
                                distance
                            }
                        };
                    
                    //update center
                    let mut new_center = pos2(pos.x - center_distance.x, pos.y - center_distance.y);

                    //check that new center is inside window
                    {
                        let size = sel.size();
                        let window_size = _frame.info().window_info.size;

                        new_center = new_center.clamp((size/2.).to_pos2(), (window_size.to_pos2() - pos2(size[0]/2. , size[1]/2.)).to_pos2());
                    }
                    
                    //update area with new center
                    let rect = Rect::from_center_size(new_center, sel.size());
                    self.center_pos = Some(new_center);
                    self.start_pos=Some(rect.min);
                    self.end_pos=Some(rect.max);
                    self.set_selected_area(Some(rect));
                }
                    
            }
            //update selected area if not in Move mode
            if self.get_status() != Status::Move {
                (new_min, new_max) = 
                    self.check_coordinates(new_min, new_max, _frame.info().window_info.size);
                self.start_pos=Some(new_max);
                self.end_pos=Some(new_min);
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
        // println!("init end pos: {:?}, {:?}", init_pos, end_pos);
        (init_pos, end_pos)

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

                    let (response, painter) = ui.allocate_painter(ctx.screen_rect().size(),egui::Sense::click_and_drag() );                  
                    ctx.set_cursor_icon(egui::CursorIcon::Crosshair);
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
                        println!("taglia");
                    }
                    ui.add_space(10.0);
                    
                    if self.is_cropping {
                        let mut pressed = false;
                        ui.with_layer_id(
                            LayerId::new(egui::Order::Foreground, Id::from("Save")),
                            |ui| {
                                    let save = ui.add(egui::Button::image(egui::Image::new(egui::include_image!("icons/save.png")).max_height(22.0)).fill(egui::Color32::TRANSPARENT));
                                    ui.add_space(10.0);
                                    let cancel = ui.add(egui::Button::image(egui::Image::new(egui::include_image!("icons/trash.png")).max_height(22.0)).fill(egui::Color32::TRANSPARENT));
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
                                                    let window = frame.info().window_info.size;
                                                    println!("window {:?}",window);
                                                    println!("centered area: {:?}", self.get_centered_area());

                                                    let rw = region.width() / self.window_scale;
                                                    let rh = region.height() / self.window_scale;
                                                    //set min centerd area as min
                                                    let min_x = region.min.x - im.min.x;
                                                    let min_y = region.min.y - im.min.y;
                                                    let rect = Rect::from_min_size(pos2(min_x, min_y), vec2(rw, rh));

                                                    //scale width and length
                                                    // let wl = window[1];
                                                    // let il = im.height();
                                                    // let rl = region.height();
                                                    
                                                    // let ww = window[0];
                                                    // let iw = im.width();
                                                    // let rw = region.width();

                                                    // let scale_width = ((rw * iw) / ww) / self.window_scale;
                                                    // let scale_height = ((rl * il) / wl) / self.window_scale;

                                                    
                                                    // let rect = Rect::from_min_size(pos2(min_x, min_y), Vec2::new(scale_width, scale_height));
        
                                                    // let scale_rect = Rect::from_min_size(pos2(scale_rect_x, scale_rect_y), Vec2::new(scale_width, scale_height));
                                                    println!("scaled rect: {:?}", rect);
                                                    self.color_image=Some(self.color_image.clone().unwrap().region(&rect, pixels_per_point));
                                                    self.handle= Some(ctx.load_texture("handle", self.color_image.clone().unwrap(),egui::TextureOptions::LINEAR));
                                                    // let sized_image = egui::load::SizedTexture::new(self.handle.clone().unwrap().id(), egui::vec2(self.color_image.clone().unwrap().size[0] as f32, self.color_image.clone().unwrap().size[1] as f32));
                                                    let sized_image = egui::load::SizedTexture::new(self.handle.clone().unwrap().id(), egui::vec2(self.color_image.clone().unwrap().size[0] as f32, self.color_image.clone().unwrap().size[1] as f32));
                                                    self.img = Some(egui::Image::from_texture(sized_image));
                                                    self.window_scale=ctx.pixels_per_point();                 
            
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
                        
                        let pen = ui.add(egui::Button::new(egui::RichText::new("\u{270F}").size(18.0))).on_hover_text("Penna"); 
                        if pen.clicked() {
                            self.acquiring_pen = !(self.acquiring_pen);
                        }
                        if self.acquiring_pen {
                            pen.highlight();
                        }
                        self.painting.ui_control(ui);
                        
                });
            });
            egui::CentralPanel::default().show(ctx, |ui| {
                //println!("{} {}  {} {}",ui.available_rect_before_wrap().min.x,ui.available_rect_before_wrap().min.y,ui.available_rect_before_wrap().max.x,ui.available_rect_before_wrap().max.y);
                self.image_size = egui::vec2(self.img.clone().unwrap().size().unwrap().x / self.window_scale, self.img.clone().unwrap().size().unwrap().y / self.window_scale);        
                let rect = get_centre_rect(ui,self.image_size);
                self.set_selected_area(Some(rect));
                self.set_centered_area(Some(rect));
                self.img.clone().unwrap().paint_at(ui,rect);
                self.painting.ui_content(ui,self.acquiring_pen,rect);
                // println!("centered area: {:?}", self.get_centered_area());

                if self.cutting {
                    let pixels_per_point = frame.info().native_pixels_per_point;
                    let region = egui::Rect::from_two_pos(
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


                if self.is_cropping {
                    egui::Window::new("")
                    .title_bar(false)
                    .frame(egui::Frame{fill:egui::Color32::from_white_alpha(10), ..Default::default()})
                    .movable(false)
                    .constraint_to(Rect::from_min_max(rect.min, rect.max))
                    .show(ctx, |ui| {

                    let (response, mut painter) = ui.allocate_painter(self.image_size,egui::Sense::click_and_drag() );                    
                    painter.rect_filled(ctx.screen_rect(), Rounding::ZERO, Color32::from_rgba_premultiplied(0, 0, 0, 130));
                    
                    
                    
                    
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



