use eframe::{
    egui::{CentralPanel, Context, Id},
    App, Frame,
};
use egui_plot::HPlacement;

use crate::{point::CornerPoint, shape::Shape};

pub struct Application {
    shape: Shape,
}

impl App for Application {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let id = Id::new("app");

        CentralPanel::default().show(ctx, |ui| {
            // Window::new("Points")
            //     .auto_sized()
            //     .default_open(false)
            //     .default_pos((16.0, 16.0))
            //     .show(ctx, |ui| {
            //         self.curve.ui(id.with("curve_control"), ui);
            //     });

            let resp = egui_plot::Plot::new(id.with("curve_canvas"))
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
            }
        });
    }
}

impl Application {
    pub fn create(_ctx: &eframe::CreationContext<'_>) -> Box<dyn App + 'static> {
        Box::new(Self {
            shape: Shape::empty(),
        })
    }
}
