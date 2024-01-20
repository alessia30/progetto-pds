use std::vec;

use eframe::egui;
use egui::{  Rect, Color32, Stroke };

pub struct Painting {
    /// in 0-1 normalized coordinates
    pub lines: Vec<(Option<egui::epaint::RectShape>,Option<egui::epaint::CircleShape>,Vec<egui::Pos2>,Stroke)>,
    pub stroke: Stroke,
    pub temp_lines: Vec<(Option<egui::epaint::RectShape>,Option<egui::epaint::CircleShape>,Vec<egui::Pos2>,Stroke)>,
    pub drawing_pen:bool,
    pub drawing_rect:bool,
    pub drawing_circle:bool,
    start_pos: Option<egui::Pos2>,
    pub prec_area: Option<egui::Rect>,
    pub cutted_area: Option<egui::Rect>,
}

impl Default for Painting {
    fn default() -> Self {
        Self {
            lines: Default::default(),
            temp_lines: Default::default(),
            stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
            drawing_pen:false,
            drawing_rect:false,
            drawing_circle:false,
            start_pos: None,
            prec_area: None,
            cutted_area: None,
        }
    }
}

pub fn my_stroke_ui(ui: &mut crate::egui::Ui, stroke: &mut egui::epaint::Stroke, text: &str) {
    let egui::epaint::Stroke { width, color } = stroke;
        ui.add_space(10.0);
        ui.add(egui::DragValue::new(width).speed(0.1).clamp_range(0.0..=5.0))
            .on_hover_text("Spessore");
        ui.add_space(10.0);
        ui.color_edit_button_srgba(color).on_hover_text("Colore"); 
        ui.add_space(5.0);
        ui.label(text);
        // stroke preview:
        let (_id, stroke_rect) = ui.allocate_space(ui.spacing().interact_size);
        let left = stroke_rect.left_center();
        let right = stroke_rect.right_center();
        ui.painter().line_segment([left, right], (*width, *color));
}

impl Painting {
    pub fn ui_control(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        let pen = ui.add(egui::Button::new(egui::RichText::new("\u{270F}").size(18.0))).on_hover_text("Penna"); 
        ui.add_space(9.0);
        let rectangles = ui.add(egui::Button::image(egui::Image::new(egui::include_image!("icons/rect.png")).max_height(20.0))).on_hover_text("Rettangolo");
        ui.add_space(9.0);
        let circles = ui.add(egui::Button::image(egui::Image::new(egui::include_image!("icons/circle.png")).max_height(20.0))).on_hover_text("Cerchio");
        if pen.clicked() {
            self.drawing_pen = !(self.drawing_pen);
            self.drawing_circle=false;
            self.drawing_rect=false;
        } 
        if rectangles.clicked() {
            
            self.drawing_rect = !(self.drawing_rect);
            self.drawing_pen=false;
            self.drawing_circle=false;
        } 
        if circles.clicked() {
            self.drawing_circle = !(self.drawing_circle);
            self.drawing_pen=false;
            self.drawing_rect=false;
        }
        if self.drawing_pen {
            pen.highlight();
        } 
         if self.drawing_circle {
            circles.highlight();
        } 
         if self.drawing_rect {
            rectangles.highlight();
        }
        my_stroke_ui(ui, &mut self.stroke, "Stroke");        
        if self.lines.is_empty() {
            self.lines.push((None,None,vec![], self.stroke.clone()));
        }
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {   
                ui.visuals_mut().widgets.inactive.expansion = 2.;
                if ui.add_enabled(self.lines.len()>1, egui::Button::new(egui::RichText::new("↩").size(15.0))).on_hover_text("Annulla").clicked() {
                    let _ =self.lines.pop();
                    if let Some(line) = self.lines.pop() {
                        self.temp_lines.push(line);
                        self.lines.push((None,None,vec![], self.stroke.clone()));
                    }
                }
                ui.add_space(2.0);
                if ui.add_enabled(self.temp_lines.len() > 0, egui::Button::new(egui::RichText::new("↪").size(15.0))).on_hover_text("Ripristina").clicked() {
                    self.lines.pop();
                    self.lines.push(self.temp_lines.pop().unwrap());
                    self.lines.push((None,None,vec![], self.stroke.clone()));
                }
            });
            ui.add_space(10.0);
            if ui.add_enabled(self.lines.len() > 1,egui::Button::image(egui::Image::new(egui::include_image!("icons/trash.png")).max_height(20.0))).on_hover_text("Pulisci").clicked() {
                self.lines.clear();
                self.temp_lines.clear();      
            }
            ui.add_space(5.0);
            ui.separator();
        });
    }

    pub fn ui_content(&mut self, ui: &mut egui::Ui, rect: egui::Rect)-> egui::Response { 
       
        let (mut response, painter) = ui.allocate_painter(ui.min_size(), egui::Sense::drag());
        let to_screen;
        let mut from_screen;
        if self.prec_area.is_some(){
            let to_screen_temp = egui::emath::RectTransform::from_to(
                Rect::from_min_size(egui::Pos2::ZERO, self.prec_area.unwrap().square_proportions()),
                 self.prec_area.unwrap(),
            );
            from_screen = to_screen_temp.inverse();
            let prec_area_min =from_screen * self.prec_area.unwrap().min;
            let cutted_area_min =  from_screen * self.cutted_area.unwrap().min;
            let trasl = egui::vec2(prec_area_min.x - cutted_area_min.x, prec_area_min.y - cutted_area_min.y);
            to_screen = egui::emath::RectTransform::from_to(
                Rect::from_min_size(egui::Pos2::ZERO, self.cutted_area.unwrap().square_proportions()),
                Rect::from_min_size(self.prec_area.unwrap().min,self.cutted_area.unwrap().size()),
            );
            from_screen = to_screen.inverse();
            for line in &mut self.lines {
                if let Some(rectangle) = &mut line.0 {
                    rectangle.rect.min.x += trasl.x;
                    rectangle.rect.min.y += trasl.y;
                    rectangle.rect.min = to_screen_temp * rectangle.rect.min;
                    rectangle.rect.min = from_screen * rectangle.rect.min;

                    rectangle.rect.max.x += trasl.x;
                    rectangle.rect.max.y += trasl.y;
                    rectangle.rect.max = to_screen_temp * rectangle.rect.max;
                    rectangle.rect.max = from_screen * rectangle.rect.max;

                } else if let Some(circle) = &mut line.1 {
                    circle.center.x += trasl.x;
                    circle.center.y += trasl.y;
                    let r_pos = egui::pos2(circle.center.x + circle.radius, circle.center.y);
                    let r_pos_transf = from_screen * (to_screen_temp * r_pos);
                    circle.center = to_screen_temp * circle.center;
                    circle.center = from_screen * circle.center;
                    circle.radius = circle.center.distance(r_pos_transf);

                } else {
                    for pos in &mut line.2 {
                        pos.x += trasl.x;
                        pos.y += trasl.y;
                        *pos= to_screen_temp * *pos;
                        *pos= from_screen * *pos;
                    }
                }
            }
            self.prec_area=None;
        } else {
            to_screen = egui::emath::RectTransform::from_to(
                Rect::from_min_size(egui::Pos2::ZERO, rect.square_proportions()),
                rect,
            );
            from_screen = to_screen.inverse();
        }
         
        if self.lines.is_empty() {
            self.lines.push((None,None,vec![], self.stroke.clone()));
        }
        let current_line = self.lines.last_mut().unwrap();
        if self.drawing_pen {   
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                let canvas_pos = from_screen * pointer_pos;
                if current_line.2.last() != Some(&canvas_pos) {
                    current_line.2.push(canvas_pos);
                    current_line.3 = self.stroke.clone();
                    response.mark_changed();
                }
            } else if !current_line.2.is_empty() { 
                self.lines.push((None,None,vec![], self.stroke.clone()));
                self.temp_lines.clear();
                response.mark_changed();
            }
        } else if self.drawing_circle {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                let canvas_pos = from_screen * pointer_pos;
                if self.start_pos.is_none() {
                    self.start_pos=Some(canvas_pos);
                } else {
                    current_line.1=Some(egui::epaint::CircleShape::stroke(self.start_pos.unwrap(), self.start_pos.unwrap().distance(canvas_pos), self.stroke.clone()));
                    response.mark_changed();
                }
            } else if current_line.1.is_some() { 
                self.start_pos=None;
                self.lines.push((None,None,vec![], self.stroke.clone()));
                self.temp_lines.clear();
                response.mark_changed();
            }
        } else if self.drawing_rect {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                let canvas_pos = from_screen * pointer_pos;
                if self.start_pos.is_none() {
                    self.start_pos=Some(canvas_pos);
                } else {
                    current_line.0=Some( egui::epaint::RectShape::new(egui::Rect::from_two_pos(self.start_pos.unwrap(), canvas_pos), egui::Rounding::ZERO, egui::Color32::TRANSPARENT, self.stroke.clone()));
                    response.mark_changed();
                }
            } else if current_line.0.is_some() { 
                self.start_pos=None;
                self.lines.push((None,None,vec![], self.stroke.clone()));
                self.temp_lines.clear();
                response.mark_changed();
            }
        }
        
        // Disegna le linee
        let shapes = self
                .lines
                .iter()
                .filter(|(rettangolo,cerchio ,line, _)| line.len() >= 2 || rettangolo.is_some() || cerchio.is_some())
                .map(|(rettangolo,cerchio,line, stroke)| {
                    if let Some(mut rettangolo) = rettangolo {
                        rettangolo.rect = egui::Rect::from_two_pos(to_screen * rettangolo.rect.min, to_screen * rettangolo.rect.max);
                        egui::Shape::Rect(rettangolo)
                    } else if let Some(mut cerchio) = cerchio {
                        let last_pos = to_screen * egui::Pos2 { x: cerchio.center.x, y: cerchio.center.y+cerchio.radius };
                        cerchio.center=to_screen * cerchio.center;
                        cerchio.radius=cerchio.center.distance(last_pos);
                        egui::Shape::Circle(cerchio)
                    }else{
                        let points: Vec<egui::Pos2> = line.iter().map(|p| to_screen * *p).collect();
                        egui::Shape::line(points, *stroke)
                    }
                });
            painter.extend(shapes.clone());        
        response
        }

        pub fn set_false(&mut self){
            self.lines.clear();
            self.temp_lines.clear();
            self.drawing_circle=false;
            self.drawing_pen=false;
            self.drawing_rect=false;
        }
}