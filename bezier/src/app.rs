use eframe::{
    egui::{CentralPanel, Context, Id, Key, Window},
    epaint::Color32,
    App, Frame,
};
use egui_plot::{HPlacement, MarkerShape, Plot};

use crate::{
    option::PointPlotOption,
    point::{CornerPoint, PlotPointExt, SmoothPoint},
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
                    self.shape.controls(ui, id.with("shape_data"));
                });

            if ctx.input(|i| i.key_released(Key::C)) {
                self.shape.toggle_close();
            }

            let resp = Plot::new(id.with("shape_canvas"))
                .data_aspect(1.0)
                .include_x(-50.0)
                .include_x(50.0)
                .include_y(-30.0)
                .include_y(10.0)
                .y_axis_width(3)
                .y_axis_position(HPlacement::Right)
                .show(ui, |plot| {
                    self.shape.plot(plot);

                    if let Some(pos) = ctx.pointer_hover_pos() {
                        let point = plot.transform().value_from_position(pos);
                        if let Some((_, sp, _)) = self.shape.nearest_point_on_segment(&point) {
                            sp.plot(plot, PointPlotOption {
                                size: 10.0,
                                mark: MarkerShape::Circle,
                                color: Color32::KHAKI,
                            })
                        }
                    }
                });

            self.shape.interact(ui, id, resp);
        });
    }
}

impl Application {
    pub fn create(_ctx: &eframe::CreationContext<'_>) -> Box<dyn App + 'static> {
        let shape = [
            CornerPoint::new([-40.0, 0.0].into())
                .with_out_ctrl([-20.0, -20.0].into())
                .into(),
            SmoothPoint::horizontal([0.0, -20.0].into(), 10.0, 10.0).into(),
            CornerPoint::new([40.0, 0.0].into())
                .with_in_ctrl([20.0, -20.0].into())
                .into(),
        ]
        .into_iter()
        .collect();
        Box::new(Self { shape })
    }
}
