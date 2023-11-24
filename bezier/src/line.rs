use eframe::epaint::Color32;
use egui_plot::{PlotPoint, PlotPoints, PlotUi};

pub struct LineSegment<'a> {
    start: &'a PlotPoint,
    end: &'a PlotPoint,
}

impl<'a> LineSegment<'a> {
    pub fn new(start: &'a PlotPoint, end: &'a PlotPoint) -> Self {
        Self { start, end }
    }

    pub fn parametric_function(&self) -> impl Fn(f64) -> (f64, f64) {
        let start = *self.start;
        let end = *self.end;

        move |t| {
            let x = (1.0 - t) * start.x + t * end.x;
            let y = (1.0 - t) * start.y + t * end.y;
            (x, y)
        }
    }

    pub fn curve(&self, color: Color32, width: f32) -> egui_plot::Line {
        egui_plot::Line::new(PlotPoints::from_parametric_callback(
            self.parametric_function(),
            0.0..=1.0,
            2,
        ))
        .color(color)
        .width(width)
    }

    pub fn plot(&self, plot: &mut PlotUi, color: Color32, width: f32) {
        plot.line(self.curve(color, width))
    }
}
