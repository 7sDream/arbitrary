use std::borrow::Cow;

use dyn_stack::PodStack;
use egui_plot::{Line, PlotPoint, PlotPoints, PlotUi};
use faer_core::{Mat, Parallelism};
use faer_evd::{ComputeVectors, EvdParams};

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

    // TODO: This may not the fastest way. Alternative are:
    // 1. subdivided into line segments, find minimal distance interval, then use Newton's Method in
    //    that interval. But may not find the nearest point if segments count too less
    // 2. Improved Algebraic Method: https://inria.hal.science/file/index/docid/518379/filename/Xiao-DiaoChen2007c.pdf
    //    But the Sturm sequence seems hard to construct/understand.
    pub fn nearest_to(&self, target: &PlotPoint) -> Option<(PlotPoint, f64)> {
        let coefficients = self.distance_derivative_coefficient(target);

        // find the degree of function, remove leading zeros
        let mut degree: Option<usize> = None;
        for (i, c) in coefficients.iter().enumerate() {
            if *c != 0.0 {
                degree.replace(i);
                break;
            }
        }

        // if all coefficients is zero, or only contains constant item
        // we assume there is no nearest point
        let d = match degree {
            None | Some(5) => return None,
            Some(d) => d,
        };

        // size of companion matrix
        let size = 5 - d;

        // construct the companion matrix of polynomial
        // a_{0..n-1} is **normalized** coefficients, from low to high degree
        //
        // 0.0 0.0 ... 0.0 -a_0
        // 1.0 0.0 ... 0.0 -a_1
        // 0.0 1.0 ... 0.0 -a_2
        // ... ... ... ... ...
        // 0.0 0.0 ... 1.0 -a_{n-1}
        let mat = Mat::from_fn(size, size, |i, j| {
            if j + 1 == size {
                -coefficients[5 - i] / coefficients[d]
            } else if i == j + 1 {
                1.0
            } else {
                0.0
            }
        });

        // EVD decomposition to solve the origin polynomial
        let req = faer_evd::compute_evd_req::<f64>(
            size,
            ComputeVectors::No, // we do not need eigenvectors
            Parallelism::None,
            EvdParams::default(),
        )
        .ok()?;

        // TODO: make buffer poll for this maybe
        let mut buffer = vec![0u8; req.size_bytes()];
        let mut re = Mat::zeros(size, 1);
        let mut im = Mat::zeros(size, 1);

        faer_evd::compute_evd_real::<f64>(
            mat.as_ref(),
            re.as_mut(),
            im.as_mut(),
            None, // we do not need eigenvectors
            Parallelism::None,
            PodStack::new(&mut buffer),
            EvdParams::default(),
        );

        // We only need real root between (0, 1)
        let zeros = re
            .col_as_slice(0)
            .iter()
            .copied()
            .zip(im.col_as_slice(0).iter().copied())
            .filter(|(re, im)| 0.0 < *re && *re < 1.0 && *im == 0.0)
            .map(|(re, _)| re);

        // Compare all found roots, find the minimal distance
        let f = self.parametric_function();

        zeros
            .map(|t| {
                let (x, y) = f(t);
                let d = (x - target.x).powi(2) + (y - target.y).powi(2);
                ([x, y].into(), d)
            })
            .min_by(|(_, d), (_, d2)| d.total_cmp(d2))
            .map(|(p, d)| (p, d.sqrt()))
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
