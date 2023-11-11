#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{egui, egui_glow::Painter, glow::LINEAR};
use screenshots::{Screen};
use egui::{Order, PointerButton, RawInput, Vec2, Widget};
use image::EncodableLayout;


struct Stat {
    is_drawing: bool,
    start_pos: egui::Pos2,
    end_pos: egui::Pos2,
}
fn main() {
    let is_drawing= false;
    let mut state=Stat{
        is_drawing:false,
        start_pos:egui::Pos2::new(0.0,0.0),
        end_pos:egui::Pos2::new(0.0,0.0),
    };
    
    /*let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(400.0, 240.0)),
        ..Default::default()
    };
    //let mut ctx = egui::Context::
    let _ = eframe::run_simple_native("My egui App", options, move |ctx, frame| {
        egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
            
            ui.horizontal(|ui| {
                
                if ui.button("\u{2795} Nuovo").on_hover_text("Nuova Cattura").clicked() {
                    let _=frame.close();
                   
                    /*egui::Window::new("Modifica shortcut").show(&ctx, |ui| {
                        // Qui puoi rilevare gli eventi di input e aggiornare la variabile key
                        for event in ui.input(|i| i.events.clone()) {
                            match event {
                                egui::Event::PointerButton { pos,button: egui::Event::PointerButton::Primary,pressed: true,modifiers } => {
                                    if state.is_drawing {
                                        state.end_pos = pos;
                                    }
                                }
                            }    
                        }
                    });*/
                  //  frame.set_minimized(true);
                    /*
                    
                    egui::Window::new("").frame(egui::Frame::none()
                    .fill(egui::Color32::TRANSPARENT))
                        
                        .show(&ctx, |ui| {
                            for event in ui.input(|i| i.events.clone()) {
                                match event {
                                    egui::Event::PointerButton { pos,button:PointerButton::Primary,pressed: true,modifiers } => {
                                        
                                        if state.is_drawing {
                                            state.end_pos = pos;
                                        }
                                    }
                                    _ => {}
                                }    
                            }
                     });*/
                }
            });
        });
    });*/
    let options2 = eframe::NativeOptions {
       // mouse_passthrough:true,
       // transparent:true,
        //fullscreen:true,
        //decorated:false,
        //maximized:true,
       
        //always_on_top:true,
        ..Default::default()
    };
    
    let _ =eframe::run_native(
        "transp. wind",
        options2,
        Box::new(|cc|{
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(MyApp::new(cc))
            })
        );
}


struct MyApp<'a>{
    acquiring:bool,
    fullscreen:bool,
    img:Option<egui::Image<'a>>,
    handle:Option<egui::TextureHandle>,
    window_size: egui::Vec2,
}

impl MyApp<'_>{
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        
        Self{acquiring:false,fullscreen:false,img:None,handle:None,window_size:egui::vec2(0.0, 0.0)}
    }
}

impl eframe::App for MyApp<'_>{
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        let rgba = egui::Rgba::from_white_alpha(0.0);
        rgba.to_array()
    }
    
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::Window::new("").title_bar(false).frame(egui::Frame{fill:egui::Color32::from_white_alpha(10), ..Default::default()})
        .show(&ctx, |ui| {
        });
         println!("ciao");
        //ctx.set_visuals(egui::Visuals{window_fill:egui::Color32::from_rgba_premultiplied(0,0, 0, 0),collapsing_header_frame:true,..Default::default()});       
        if !self.acquiring{
            egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
                
                if ui.button("\u{2795} Nuovo").on_hover_text("Nuova Cattura").clicked() {
                    self.acquiring=true;
                    
                    
                   frame.set_minimized(true);
                    
                   frame.set_visible(false);
                   self.window_size=ctx.screen_rect().size();
                    
                   
                    
                    let screens = Screen::all().unwrap();                   
                        for screen in screens {
                            println!("capturer {screen:?}");
                            let image = screen.capture().unwrap();
                            println!("{} {} {}",image.width(),image.height(),image.as_bytes().len());
                            
                            let color_image=egui::ColorImage::from_rgba_unmultiplied([image.width() as usize,image.height() as usize], image.as_bytes());
                            println!("dd");
                            self.handle= Some(ctx.load_texture("handle", color_image.clone(),egui::TextureOptions::LINEAR));
                            let sized_image = egui::load::SizedTexture::new(self.handle.clone().unwrap().id(), egui::vec2(color_image.size[0] as f32, color_image.size[1] as f32));
                            println!("{} {} ",color_image.size[0],color_image.size[1]);
                            self.img = Some(egui::Image::from_texture(sized_image));
                            
                        }
                    
                }
            });
        }
        //if self.minimized
        if self.acquiring{
           
           // frame.set_fullscreen(true);
            frame.set_visible(true);
            println!("{} {} {} {}",ctx.screen_rect().min.x,ctx.screen_rect().min.y,ctx.screen_rect().max.x,ctx.screen_rect().max.y);
            
            if self.window_size != ctx.screen_rect().size(){
                self.window_size=ctx.screen_rect().size();
                self.fullscreen=true;
            }
            let scale=ctx.screen_rect().size().x/self.img.clone().unwrap().size().unwrap().x;
            let image_size = egui::vec2(self.window_size.x / scale, self.window_size.y / scale);
            println!("{}",scale);
            //if self.fullscreen{
                
            egui::Window::new("").title_bar(false)
                    .show(&ctx, |ui| {
                        ui.add(self.img.clone().unwrap().fit_to_original_size(scale));            
            });
            //}
        }
    }

}

//.frame(egui::Frame::none().fill(egui::Color32::from_white_alpha(10)))