use eframe::{
    egui::{CentralPanel, Context, Window},
    App, Frame,
};
use egui_plot::{PlotBounds, PlotPoint};

use crate::bezier::Cubic;

pub struct Application {
    curve: Cubic,
}

impl App for Application {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            let id = ui.auto_id_with("curve");
            Window::new("Points")
                .auto_sized()
                .default_open(false)
                .default_pos((16.0, 16.0))
                .show(ctx, |ui| {
                    self.curve.controls(id, ui);
                });
            self.curve.plot(
                id,
                PlotBounds::from_min_max([-50.0, -30.0], [50.0, 10.0]),
                ui,
            );
        });
    }
}

impl Application {
    pub fn create(_ctx: &eframe::CreationContext<'_>) -> Box<dyn App + 'static> {
        Box::new(Self {
            curve: Cubic::new(
                PlotPoint { x: -40.0, y: 0.0 },
                PlotPoint { x: 40.0, y: 0.0 },
                PlotPoint { x: -20.0, y: -20.0 },
                PlotPoint { x: 20.0, y: -20.0 },
            ),
        })
    }
}
