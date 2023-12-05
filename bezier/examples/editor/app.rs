use bezier::{CornerPoint, Shape, SmoothPoint};
use eframe::{
    egui::{menu, CentralPanel, Context, Id, Key, TopBottomPanel, Ui, ViewportCommand},
    App, Frame,
};
use egui_plot::{HPlacement, Plot};

use crate::{
    configure,
    interact::ShapeInteract,
    plot::plot_shape,
    point::Point,
    window::{ConfigureWindow, FloatWindow, ShapeDataWindow},
};

pub struct Application {
    id: Id,
    shape: Shape<Point>,
}

impl Application {
    fn configure_window_id(&self) -> Id {
        self.id.with("configure-window")
    }

    fn shape_window_id(&self) -> Id {
        self.id.with("shape-window")
    }

    fn menu_bar(&self, ui: &mut Ui) {
        ui.menu_button("File", |ui| {
            if ui.button("Import...").clicked() {
                // TODO
            }

            if ui.button("Export...").clicked() {
                //  TODO
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                if ui.button("Exit").clicked() {
                    ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                    ui.close_menu();
                }
            }
        });

        ui.menu_button("Edit", |ui| {
            if ui.button("Shape Data...").clicked() {
                ShapeDataWindow::open(ui, self.shape_window_id());
                ui.close_menu();
            }
            if ui.button("Configure...").clicked() {
                ConfigureWindow::open(ui, self.configure_window_id());
                ui.close_menu();
            }
        });

        ui.menu_button("View", |ui| {
            ConfigureWindow::tab_view(ui, &mut configure::write().view);
        });
    }
}

impl App for Application {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        TopBottomPanel::top(self.id.with("top-panel")).show(ctx, |ui| {
            menu::bar(ui, |ui| {
                self.menu_bar(ui);
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ConfigureWindow::new(ui, self.configure_window_id()).show(ui, &mut None);
            ShapeDataWindow::new(ui, self.shape_window_id()).show(ui, &mut self.shape);

            if ctx.input(|i| i.key_released(Key::C)) {
                self.shape.toggle_close();
            }

            let show_grid = configure::read().view.grid;

            let response = Plot::new(self.id.with("shape_canvas"))
                .data_aspect(1.0)
                .include_x(-50.0)
                .include_x(50.0)
                .include_y(-30.0)
                .include_y(10.0)
                .show_grid(show_grid)
                .show_x(false)
                .show_y(false)
                .allow_drag(ui.input(|i| i.key_down(Key::Space)))
                .y_axis_width(3)
                .y_axis_position(HPlacement::Right)
                .show(ui, |ui| {
                    plot_shape(&self.shape, ui);
                });

            ShapeInteract::new(&mut self.shape).interact(
                ui,
                self.id.with("shape_interact"),
                &response,
            );
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
        Box::new(Self {
            id: Id::new("app"),
            shape,
        })
    }
}
