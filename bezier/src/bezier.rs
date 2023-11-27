use std::borrow::Cow;

use egui_plot::{Line, PlotPoint, PlotPoints, PlotUi};

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

        let mut degree: Option<usize> = None;
        for (i, c) in coefficients.iter().enumerate() {
            if *c != 0.0 {
                degree.replace(i);
                break;
            }
        }

        let d = match degree {
            None | Some(5) => return None,
            Some(d) => d,
        };

        let mat_size = 5 - d;

        let mat = faer_core::Mat::from_fn(mat_size, mat_size, |i, j| {
            if j + 1 == mat_size {
                -coefficients[5 - i] / coefficients[d]
            } else if i == j + 1 {
                1.0
            } else {
                0.0
            }
        });

        let req = faer_evd::compute_evd_req::<f64>(
            mat_size,
            faer_evd::ComputeVectors::Yes,
            faer_core::Parallelism::None,
            faer_evd::EvdParams::default(),
        )
        .ok()?;

        let mut buffer = vec![0u8; req.size_bytes()];
        let mut s_re = faer_core::Mat::zeros(mat_size, 1);
        let mut s_im = faer_core::Mat::zeros(mat_size, 1);
        let mut u = faer_core::Mat::zeros(mat_size, mat_size);
        faer_evd::compute_evd_real::<f64>(
            mat.as_ref(),
            s_re.as_mut(),
            s_im.as_mut(),
            Some(u.as_mut()),
            faer_core::Parallelism::None,
            dyn_stack::PodStack::new(&mut buffer),
            faer_evd::EvdParams::default(),
        );

        let zeros = s_re
            .col_as_slice(0)
            .iter()
            .copied()
            .zip(s_im.col_as_slice(0).iter().copied())
            .filter(|(_, im)| *im == 0.0)
            .map(|(re, _)| re);

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
