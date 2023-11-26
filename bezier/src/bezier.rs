use std::borrow::Cow;

use egui_plot::{Line, PlotPoint, PlotPoints, PlotUi};
use roots::SimpleConvergency;

use crate::option::LinePlotOption;

#[derive(Clone)]
pub struct Bezier<'a> {
    pub start: &'a PlotPoint,
    pub end: &'a PlotPoint,
    pub ctrl1: Cow<'a, PlotPoint>,
    pub ctrl2: Cow<'a, PlotPoint>,
}

impl<'a> Bezier<'a> {
    pub fn new(
        start: &'a PlotPoint, ctrl1: Cow<'a, PlotPoint>, ctrl2: Cow<'a, PlotPoint>,
        end: &'a PlotPoint,
    ) -> Self {
        Self {
            start,
            end,
            ctrl1,
            ctrl2,
        }
    }

    pub fn new_quad<'b>(start: &'a PlotPoint, ctrl: &'b PlotPoint, end: &'a PlotPoint) -> Self {
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

    // fn parametric_function(&self) -> impl Fn(f64) -> (f64, f64) {
    //     let start = *self.start;
    //     let end = *self.end;
    //     let ctrl1 = self.ctrl1.clone().into_owned();
    //     let ctrl2 = self.ctrl2.clone().into_owned();

    //     move |t| {
    //         let nt = 1.0 - t;
    //         let x = start.x * nt.powi(3)
    //             + 3.0 * ctrl1.x * t * nt.powi(2)
    //             + 3.0 * ctrl2.x * t.powi(2) * nt
    //             + end.x * t.powi(3);
    //         let y = start.y * nt.powi(3)
    //             + 3.0 * ctrl1.y * t * nt.powi(2)
    //             + 3.0 * ctrl2.y * t.powi(2) * nt
    //             + end.y * t.powi(3);
    //         (x, y)
    //     }
    // }

    fn parametric_function_coefficient(&self) -> [PlotPoint; 4] {
        let ax = -self.start.x + 3.0 * self.ctrl1.x - 3.0 * self.ctrl2.x + self.end.x;
        let ay = -self.start.y + 3.0 * self.ctrl1.y - 3.0 * self.ctrl2.y + self.end.y;
        let bx = 3.0 * (self.start.x - 2.0 * self.ctrl1.x + self.ctrl2.x);
        let by = 3.0 * (self.start.y - 2.0 * self.ctrl1.y + self.ctrl2.y);
        let cx = -3.0 * (self.start.x - self.ctrl1.x);
        let cy = -3.0 * (self.start.y - self.ctrl1.y);
        let dx = self.start.x;
        let dy = self.start.y;

        [
            [ax, ay].into(),
            [bx, by].into(),
            [cx, cy].into(),
            [dx, dy].into(),
        ]
    }

    fn parametric_function(&self) -> impl Fn(f64) -> (f64, f64) {
        let [a, b, c, d] = self.parametric_function_coefficient();

        move |t| {
            let t2 = t * t;
            let t3 = t2 * t;
            let x = a.x * t3 + b.x * t2 + c.x * t + d.x;
            let y = a.y * t3 + b.y * t2 + c.y * t + d.y;
            (x, y)
        }
    }

    fn distance_derivative_coefficient(&self, target: &PlotPoint) -> [f64; 6] {
        let [a, b, c, d] = self.parametric_function_coefficient();

        let dtx = d.x - target.x;
        let dty = d.y - target.y;

        let ax = 3.0 * a.x * a.x;
        let bx = 5.0 * a.x * b.x;
        let cx = 4.0 * a.x * c.x + 2.0 * b.x * b.x;
        let dx = 3.0 * a.x * dtx + 3.0 * b.x * c.x;
        let ex = 2.0 * b.x * dtx + c.x * c.x;
        let fx = c.x * dtx;

        let ay = 3.0 * a.y * a.y;
        let by = 5.0 * a.y * b.y;
        let cy = 4.0 * a.y * c.y + 2.0 * b.y * b.y;
        let dy = 3.0 * a.y * dty + 3.0 * b.y * c.y;
        let ey = 2.0 * b.y * dty + c.y * c.y;
        let fy = c.y * dty;

        [ax + ay, bx + by, cx + cy, dx + dy, ex + ey, fx + fy]
    }

    // TODO: not robust enough, should find another way
    pub fn nearest_to(&self, target: &PlotPoint) -> Option<(PlotPoint, f64)> {
        let coefficients = self.distance_derivative_coefficient(target);

        let zeros: Box<dyn Iterator<Item = f64>> = if coefficients[0] == 0.0 {
            Box::new(
                #[allow(clippy::unnecessary_to_owned)] // fake positive
                roots::find_roots_quartic(
                    coefficients[1],
                    coefficients[2],
                    coefficients[3],
                    coefficients[4],
                    coefficients[5],
                )
                .as_ref()
                .to_vec()
                .into_iter(),
            )
        } else {
            let normalized = [
                coefficients[1] / coefficients[0],
                coefficients[2] / coefficients[0],
                coefficients[3] / coefficients[0],
                coefficients[4] / coefficients[0],
                coefficients[5] / coefficients[0],
            ]
            .to_vec();

            Box::new(
                roots::find_roots_sturm(&normalized, &mut SimpleConvergency::<f64> {
                    max_iter: 1024,
                    eps: 1e-6f64,
                })
                .into_iter()
                .inspect(|x| {
                    if let Err(e) = x {
                        dbg!(e);
                    }
                })
                .filter_map(Result::ok),
            )
        };

        let f = self.parametric_function();

        zeros
            .filter(|x| *x > 0.0 && *x < 1.0)
            // .chain(std::iter::once(0.0))
            // .chain(std::iter::once(1.0))
            .map(|t| {
                let (x, y) = f(t);
                let d = (x - target.x).powi(2) + (y - target.y).powi(2);
                ([x, y].into(), d.sqrt())
            })
            .min_by(|(_, d), (_, d2)| d.total_cmp(d2))
    }

    pub fn curve(&self, opt: LinePlotOption) -> Line {
        Line::new(PlotPoints::from_parametric_callback(
            self.parametric_function(),
            0.0..=1.0,
            64,
        ))
        .color(opt.color)
        .width(opt.width as f32)
    }

    pub fn plot(&self, plot: &mut PlotUi, opt: LinePlotOption) {
        plot.line(self.curve(opt))
    }
}
