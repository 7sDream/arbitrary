use std::borrow::Cow;

use eframe::epaint::Color32;
use egui_plot::{Line, PlotPoint, PlotPoints, PlotUi};

#[derive(Clone)]
pub struct Bezier<'a> {
    pub start: &'a PlotPoint,
    pub end: &'a PlotPoint,
    pub ctrl1: Cow<'a, PlotPoint>,
    pub ctrl2: Cow<'a, PlotPoint>,
}

impl<'a> Bezier<'a> {
    pub fn new(
        start: &'a PlotPoint, ctrl1: &'a PlotPoint, ctrl2: &'a PlotPoint, end: &'a PlotPoint,
    ) -> Self {
        Self {
            start,
            end,
            ctrl1: Cow::Borrowed(ctrl1),
            ctrl2: Cow::Borrowed(ctrl2),
        }
    }

    pub fn new_quad(start: &'a PlotPoint, ctrl: &'a PlotPoint, end: &'a PlotPoint) -> Self {
        fn calc(a: &PlotPoint, b: &PlotPoint) -> PlotPoint {
            let x = a.x + 2.0 * (b.x - a.x) / 3.0;
            let y = a.y + 2.0 * (b.y - a.y) / 3.0;
            PlotPoint { x, y }
        }

        Self {
            start,
            end,
            ctrl1: Cow::Owned(calc(start, ctrl)),
            ctrl2: Cow::Owned(calc(end, ctrl)),
        }
    }

    // fn polygon(&self) -> Line {
    //     Line::new(PlotPoints::Owned(vec![
    //         self.start, self.end, self.ctrl2, self.ctrl1, self.start,
    //     ]))
    //     .color(CTRL_LINK_LINE_COLOR)
    //     .width(1.0)
    // }

    fn parametric_function(&self) -> impl Fn(f64) -> (f64, f64) {
        let start = *self.start;
        let end = *self.end;
        let ctrl1 = self.ctrl1.clone().into_owned();
        let ctrl2 = self.ctrl2.clone().into_owned();

        move |t| {
            let nt = 1.0 - t;
            let x = start.x * nt.powi(3)
                + 3.0 * ctrl1.x * t * nt.powi(2)
                + 3.0 * ctrl2.x * t.powi(2) * nt
                + end.x * t.powi(3);
            let y = start.y * nt.powi(3)
                + 3.0 * ctrl1.y * t * nt.powi(2)
                + 3.0 * ctrl2.y * t.powi(2) * nt
                + end.y * t.powi(3);
            (x, y)
        }
    }

    pub fn curve(&self, color: Color32, width: f32) -> Line {
        Line::new(PlotPoints::from_parametric_callback(
            self.parametric_function(),
            0.0..=1.0,
            64,
        ))
        .color(color)
        .width(width)
    }

    pub fn plot(&self, plot: &mut PlotUi, color: Color32, width: f32) {
        plot.line(self.curve(color, width))
    }
}
