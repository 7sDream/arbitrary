use eframe::{
    egui::{CentralPanel, Context},
    App, Frame,
};
use egui_plot::PlotPoint;

use crate::bezier::Cubic;

pub struct Application {
    curve: Cubic,
}

impl App for Application {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.label("Cubic Curve");
            self.curve.ui(ui.id().with("curve"), ui)
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
