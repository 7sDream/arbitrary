use bezier::{CornerPoint, Shape, SmoothPoint};
use eframe::{
    egui::{CentralPanel, Context, Id, Key, Window},
    epaint::Color32,
    App, Frame,
};
use egui_plot::{HPlacement, Plot};

use crate::{
    interact::ShapeInteract,
    plot::{plot_point, plot_shape},
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
                    ShapeInteract::new(&mut self.shape).controls(ui, id.with("shape_data"));
                });

            if ctx.input(|i| i.key_released(Key::C)) {
                self.shape.toggle_close();
            }

            let response = Plot::new(id.with("shape_canvas"))
                .data_aspect(1.0)
                .include_x(-50.0)
                .include_x(50.0)
                .include_y(-30.0)
                .include_y(10.0)
                .show_x(false)
                .show_y(false)
                .y_axis_width(3)
                .y_axis_position(HPlacement::Right)
                .show(ui, |ui| {
                    plot_shape(&self.shape, ui);

                    if ui.response().hovered() {
                        let Some(target) = ui.pointer_coordinate() else {
                            return None;
                        };

                        let pos = ui.transform().position_from_point(&target);

                        let nearest = ShapeInteract::new(&mut self.shape)
                            .snap_to_curve_with_radius(&target, pos, ui.transform(), 12.0);

                        if let Some(ref n) = nearest {
                            plot_point(&n.point, ui, crate::option::PointPlotOption {
                                mark: egui_plot::MarkerShape::Diamond,
                                size: 8.0,
                                color: Color32::BLACK,
                            });
                        }

                        if ui.response().clicked() {
                            return Some((target, nearest));
                        }
                    }

                    None
                });

            ShapeInteract::new(&mut self.shape).interact(ui, id, response);
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