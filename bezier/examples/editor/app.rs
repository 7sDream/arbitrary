use bezier::{CornerPoint, Shape, SmoothPoint};
use eframe::{
    egui::{CentralPanel, Context, Id, Key, Window},
    epaint::Color32,
    App, Frame,
};
use egui_plot::{HPlacement, Plot};

use crate::{
    configure::{self, configure_window, PointPlotConfig},
    interact::ShapeInteract,
    plot::{plot_point, plot_shape},
    point::Point,
};

pub struct Application {
    shape: Shape<Point>,
}

impl App for Application {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let id = Id::new("app");

        CentralPanel::default().show(ctx, |ui| {
            configure_window(ctx);

            if configure::read().windows.shape_data {
                Window::new("Shape Data")
                    .id(Id::new("shape_data_window"))
                    .auto_sized()
                    .default_open(false)
                    .show(ctx, |ui| {
                        ShapeInteract::new(&mut self.shape).controls(ui, id.with("shape_data"));
                    });
            }

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

                        let target = Point(target);

                        let nearest = ShapeInteract::new(&mut self.shape)
                            .snap_to_curve_with_radius(&target, pos, ui.transform(), 12.0);

                        if let Some(ref n) = nearest {
                            plot_point(&n.point, ui, &PointPlotConfig {
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
            CornerPoint::new(Point::new(-40.0, 0.0))
                .with_out_ctrl(Point::new(-20.0, -20.0))
                .into(),
            SmoothPoint::horizontal(Point::new(0.0, -20.0), 10.0, 10.0).into(),
            CornerPoint::new(Point::new(40.0, 0.0))
                .with_in_ctrl(Point::new(20.0, -20.0))
                .into(),
        ]
        .into_iter()
        .collect();
        Box::new(Self { shape })
    }
}
