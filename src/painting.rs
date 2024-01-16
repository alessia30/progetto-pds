use eframe::egui;
use egui::{  Rect, Color32, Stroke };

pub struct Painting {
    /// in 0-1 normalized coordinates
    pub lines: Vec<(Vec<egui::Pos2>,Stroke)>,
    pub stroke: Stroke,
    pub temp_lines: Vec<(Vec<egui::Pos2>,Stroke)>,
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
    ui.vertical_centered(|ui| {
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
    });
}

impl Painting {
    pub fn ui_control(&mut self, ui: &mut egui::Ui) {
        my_stroke_ui(ui, &mut self.stroke, "Stroke");        
        if self.lines.is_empty() {
            self.lines.push((vec![], self.stroke.clone()));
        }
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {   
                ui.visuals_mut().widgets.inactive.expansion = 2.;
                if ui.add_enabled(self.lines.len()>1, egui::Button::new(egui::RichText::new("↩").size(15.0))).on_hover_text("Annulla").clicked() {
                    let _ =self.lines.pop();
                    if let Some(line) = self.lines.pop() {
                        self.temp_lines.push(line);
                        self.lines.push((vec![], self.stroke.clone()));
                    }
                }
                ui.add_space(2.0);
                if ui.add_enabled(self.temp_lines.len() > 0, egui::Button::new(egui::RichText::new("↪").size(15.0))).on_hover_text("Ripristina").clicked() {
                    self.lines.pop();
                    self.lines.push(self.temp_lines.pop().unwrap());
                    self.lines.push((vec![], self.stroke.clone()));
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

    pub fn ui_content(&mut self, ui: &mut egui::Ui, drawing: bool,rect: egui::Rect)-> egui::Response { 
       
        let (mut response, painter) = ui.allocate_painter(ui.min_size(), egui::Sense::drag());
        let to_screen = egui::emath::RectTransform::from_to(
            Rect::from_min_size(egui::Pos2::ZERO, rect.square_proportions()),
            rect,
        );
        let from_screen = to_screen.inverse(); 

        if drawing {   
            if self.lines.is_empty() {
                self.lines.push((vec![], self.stroke.clone()));
            }
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