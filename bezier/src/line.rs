use egui_plot::{PlotPoint, PlotPoints, PlotUi};
use roots::Roots;

use crate::option::LinePlotOption;

pub struct LineSegment<'a> {
    start: &'a PlotPoint,
    end: &'a PlotPoint,
}

impl<'a> LineSegment<'a> {
    pub fn new(start: &'a PlotPoint, end: &'a PlotPoint) -> Self {
        Self { start, end }
    }

    pub fn parametric_function_coefficient(&self) -> [PlotPoint; 2] {
        [
            [self.end.x - self.start.x, self.end.y - self.start.y].into(),
            *self.start,
        ]
    }

    pub fn parametric_function(&self) -> impl Fn(f64) -> (f64, f64) {
        let [a, b] = self.parametric_function_coefficient();

        move |t| {
            let x = a.x * t + b.x;
            let y = a.y * t + b.y;
            (x, y)
        }
    }

    fn distance_derivative_coefficient(&self, target: &PlotPoint) -> [f64; 2] {
        let [a, b] = self.parametric_function_coefficient();

        let ax = a.x * a.x;
        let ay = a.y * a.y;

        let bx = (b.x - target.x) * a.x;
        let by = (b.y - target.y) * a.y;

        [ax + ay, bx + by]
    }

    pub fn nearest_to(&self, target: &PlotPoint) -> Option<(PlotPoint, f64)> {
        let [a, b] = self.distance_derivative_coefficient(target);
        let t = roots::find_roots_linear(a, b);
        match t {
            Roots::One([t]) if 0.0 < t && t < 1.0 => {
                let f = self.parametric_function();
                let (x, y) = f(t);
                let d = (x - target.x).powi(2) + (y - target.y).powi(2);
                Some(([x, y].into(), d.sqrt()))
            }
            _ => None,
        }
    }

    pub fn curve(&self, opt: LinePlotOption) -> egui_plot::Line {
        egui_plot::Line::new(PlotPoints::from_parametric_callback(
            self.parametric_function(),
            0.0..=1.0,
            2,
        ))
        .color(opt.color)
        .width(opt.width as f32)
    }

    pub fn plot(&self, plot: &mut PlotUi, opt: LinePlotOption) {
        plot.line(self.curve(opt))
    }
}
