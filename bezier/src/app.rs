use eframe::{
    egui::{CentralPanel, Context, Id, Key, Window},
    App, Frame,
};
use egui_plot::HPlacement;

use crate::{
    point::{CornerPoint, SmoothPoint},
    shape::Shape,
};

pub struct Application {
    shape: Shape,
}

impl App for Application {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let id = Id::new("app");

        CentralPanel::default().show(ctx, |ui| {
            Window::new("Shape Data")
                .auto_sized()
                .default_open(false)
                .default_pos((16.0, 16.0))
                .show(ctx, |ui| {
                    self.shape.ui(ui, id.with("shape_data"));
                });

            if ctx.input(|i| i.key_released(Key::C)) {
                self.shape.toggle_close();
            }

            let resp = egui_plot::Plot::new(id.with("shape_canvas"))
                .data_aspect(1.0)
                .include_x(-50.0)
                .include_x(50.0)
                .include_y(-30.0)
                .include_y(10.0)
                .y_axis_width(3)
                .y_axis_position(HPlacement::Right)
                .show(ui, |plot| self.shape.plot(plot));

            if resp.response.clicked() {
                if let Some(pos) = resp.response.interact_pointer_pos() {
                    let point = resp.transform.value_from_position(pos);
                    self.shape.push(CornerPoint::new(point))
                }
            } else {
                self.shape
                    .interact(ui, id.with("shape_interact"), resp.transform);
            }
        });
    }
}

impl Application {
    pub fn create(_ctx: &eframe::CreationContext<'_>) -> Box<dyn App + 'static> {
        let shape = [
            CornerPoint::new([-40.0, 0.0].into())
                .with_out_ctrl([-20.0, -20.0].into())
                .into(),
            SmoothPoint::horizontal([0.0, -20.0].into(), 10.0).into(),
            CornerPoint::new([40.0, 0.0].into())
                .with_in_ctrl([20.0, -20.0].into())
                .into(),
        ]
        .into_iter()
        .collect();
        Box::new(Self { shape })
    }
}
