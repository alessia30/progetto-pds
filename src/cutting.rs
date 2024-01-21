use eframe::egui;
use egui::{Vec2, Rect, Color32, LayerId, Id, CursorIcon, pos2, Context, Painter, Pos2, Order };
use crate::myapp::MyApp;

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

impl MyApp<'_>{
    pub fn get_selected_area (&self) -> Option<Rect> {
        self.selected_area
    }
    
    pub fn get_status (&self) -> Status {
        self.status
    }

    pub fn get_centered_area (&self) -> Option<Rect> {
        self.centered_area
    }
    
    pub fn set_selected_area(&mut self, new_area: Option<Rect>) {
        self.selected_area = new_area;
    }

    pub fn set_centered_area(&mut self, new_area: Option<Rect>) {
        self.centered_area = new_area;
    }
    
    pub fn set_status(&mut self, new_status: Status) {
        self.status = new_status;
    }

    pub fn scale_selection(&mut self, ctx: &Context, _frame: &mut eframe::Frame, painter: &mut Painter) {
        let sel = self
        .get_selected_area()
        .expect("Selected area must be Some");
        let status = self.get_status();
        //println!("{:?}", status);

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

    pub fn update_area (&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame, pos: Pos2, status: Status, _painter: &mut Painter) {
        let sel = self.get_selected_area().expect("Selected area must be some when updating it");
        let mut new_min = sel.min; 
        let mut new_max = sel.max;
        //println!("pos: {:?}", pos);
        let image_rect = self.get_centered_area().unwrap();
        if ctx.input(|i| i.pointer.primary_down()) {
            match self.get_status() {
                Status::None => self.set_status(status), //set status if enters with None
                Status::Select => unreachable!("Sould not be un select mode during update"),
                Status::TopLeft => {
                    if pos.x < image_rect.min.x {
                        new_min.x = image_rect.min.x 
                    } else {
                        new_min.x = pos.x;
                    }
                    if pos.y < image_rect.min.y {
                        new_min.y = image_rect.min.y; 
                    } else {
                        new_min.y = pos.y;
                    }
                },
                Status::TopMid => {
                    if pos.y < image_rect.min.y {
                        new_min.y = image_rect.min.y; 
                    } else {
                        new_min.y = pos.y;
                    }
                },
                Status::TopRight => {
                    if pos.y < image_rect.min.y {
                        new_min.y = image_rect.min.y 
                    } else {
                        new_min.y = pos.y;
                    }
                    if pos.x > image_rect.max.x {
                        new_max.x = image_rect.max.x; 
                    } else {
                        new_max.x = pos.x;
                    }
                },
                Status::MidLeft => {
                    if pos.x < image_rect.min.x {
                        new_min.x = image_rect.min.x 
                    } else {
                        new_min.x = pos.x;
                    }
                },
                Status::MidRight => {
                    if pos.x > image_rect.max.x {
                        new_max.x = image_rect.max.x; 
                    } else {
                        new_max.x = pos.x;
                    }
                },
                Status::BotLeft => {
                    if pos.x < image_rect.min.x {
                        new_min.x = image_rect.min.x;
                    } else {
                        new_min.x = pos.x;
                    }
                    if pos.y > image_rect.max.y {
                        new_max.y = image_rect.max.y; 
                    } else {
                        new_max.y = pos.y;
                    }
                },
                Status::BotMid => {
                    if pos.y > image_rect.max.y {
                        new_max.y = image_rect.max.y; 
                    } else {
                        new_max.y = pos.y;
                    }
                },
                Status::BotRight => {
                    if pos.x > image_rect.max.x {
                        new_max.x = image_rect.max.x 
                    } else {
                        new_max.x = pos.x;
                    }
                    if pos.y > image_rect.max.y {
                        new_max.y = image_rect.max.y; 
                    } else {
                        new_max.y = pos.y;
                    }
                },
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

                    //check that new center is inside rect
                    {
                        let size = sel.size();
                        new_center = new_center.clamp(image_rect.min + (size/2.) , (image_rect.max - pos2(size[0]/2. , size[1]/2.)).to_pos2());
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

    pub fn check_coordinates(&mut self, start: Pos2, end: Pos2, window_size: Vec2) -> (Pos2, Pos2) {
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