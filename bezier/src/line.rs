use eframe::egui::Ui;
use egui_plot::{PlotPoint, PlotPoints, PlotTransform, PlotUi, Points};

use crate::color::CURVE_COLOR;

pub struct Line {
    start: PlotPoint,
    end: PlotPoint,
}

impl Line {
    pub fn new(start: PlotPoint, end: PlotPoint) -> Self {
        Self { start, end }
    }

    pub fn parametric_function(&self) -> impl Fn(f64) -> (f64, f64) {
        let Self { start, end } = *self;

        move |t| {
            let x = (1.0 - t) * start.x + t * end.x;
            let y = (1.0 - t) * start.y + t * end.y;
            (x, y)
        }
    }

    pub fn curve(&self) -> egui_plot::Line {
        egui_plot::Line::new(PlotPoints::from_parametric_callback(
            self.parametric_function(),
            0.0..=1.0,
            2,
        ))
        .color(CURVE_COLOR)
        .width(2.0)
    }

    pub fn plot(&self, plot: &mut PlotUi) {
        plot.line(self.curve());
    }

    pub fn ui(&self, ui: &mut Ui) {}

    pub fn drag(&self, transform: PlotTransform, ui: &mut Ui) {}
}
