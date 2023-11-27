use dyn_stack::PodStack;
use faer_core::{Mat, Parallelism};
use faer_evd::{ComputeVectors, EvdParams};

use super::{Nearest, Point};

pub struct Bezier {
    pub start: Point,
    pub ctrl1: Point,
    pub ctrl2: Point,
    pub end: Point,
}

impl Bezier {
    pub fn new(start: Point, ctrl1: Point, ctrl2: Point, end: Point) -> Self {
        Self {
            start,
            end,
            ctrl1,
            ctrl2,
        }
    }

    pub fn new_quad(start: Point, ctrl: Point, end: Point) -> Self {
        fn calc(a: &Point, b: &Point) -> Point {
            let x = a.0 + 2.0 * (b.0 - a.0) / 3.0;
            let y = a.1 + 2.0 * (b.1 - a.1) / 3.0;
            (x, y)
        }

        Self {
            start,
            end,
            ctrl1: calc(&start, &ctrl),
            ctrl2: calc(&end, &ctrl),
        }
    }

    fn parametric_function_coefficient(&self) -> [Point; 4] {
        let ax = -self.start.0 + 3.0 * self.ctrl1.0 - 3.0 * self.ctrl2.0 + self.end.0;
        let ay = -self.start.1 + 3.0 * self.ctrl1.1 - 3.0 * self.ctrl2.1 + self.end.1;
        let bx = 3.0 * (self.start.0 - 2.0 * self.ctrl1.0 + self.ctrl2.0);
        let by = 3.0 * (self.start.1 - 2.0 * self.ctrl1.1 + self.ctrl2.1);
        let cx = -3.0 * (self.start.0 - self.ctrl1.0);
        let cy = -3.0 * (self.start.1 - self.ctrl1.1);
        let dx = self.start.0;
        let dy = self.start.1;

        [(ax, ay), (bx, by), (cx, cy), (dx, dy)]
    }

    pub fn parametric_function(&self) -> impl Fn(f64) -> Point {
        let [a, b, c, d] = self.parametric_function_coefficient();

        move |t| {
            let t2 = t * t;
            let t3 = t2 * t;
            let x = a.0 * t3 + b.0 * t2 + c.0 * t + d.0;
            let y = a.1 * t3 + b.1 * t2 + c.1 * t + d.1;
            (x, y)
        }
    }

    pub fn at(&self, t: f64) -> Point {
        self.parametric_function()(t)
    }

    fn distance_derivative_coefficient(&self, target: &Point) -> [f64; 6] {
        let [a, b, c, d] = self.parametric_function_coefficient();

        let dtx = d.0 - target.0;
        let dty = d.1 - target.1;

        let ax = 3.0 * a.0 * a.0;
        let bx = 5.0 * a.0 * b.0;
        let cx = 4.0 * a.0 * c.0 + 2.0 * b.0 * b.0;
        let dx = 3.0 * a.0 * dtx + 3.0 * b.0 * c.0;
        let ex = 2.0 * b.0 * dtx + c.0 * c.0;
        let fx = c.0 * dtx;

        let ay = 3.0 * a.1 * a.1;
        let by = 5.0 * a.1 * b.1;
        let cy = 4.0 * a.1 * c.1 + 2.0 * b.1 * b.1;
        let dy = 3.0 * a.1 * dty + 3.0 * b.1 * c.1;
        let ey = 2.0 * b.1 * dty + c.1 * c.1;
        let fy = c.1 * dty;

        [ax + ay, bx + by, cx + cy, dx + dy, ex + ey, fx + fy]
    }

    // TODO: This may not the fastest way. Alternative are:
    // 1. subdivided into line segments, find minimal distance interval, then use Newton's Method in
    //    that interval. But may not find the nearest point if segments count too less
    // 2. Improved Algebraic Method: https://inria.hal.science/file/index/docid/518379/filename/Xiao-DiaoChen2007c.pdf
    //    But the Sturm sequence seems hard to construct/understand.
    fn solve_poly(coefficients: [f64; 6]) -> Option<(Mat<f64>, Mat<f64>)> {
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
            // TODO: figure out why this set to No will cause wasm32 OOM when calculating
            ComputeVectors::Yes,
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

        Some((re, im))
    }

    /// Calculate the nearest point on the segment to a provided target point.
    pub fn nearest_to(&self, target: &Point, allow_endpoint: bool) -> Option<Nearest> {
        let coefficients = self.distance_derivative_coefficient(target);

        if let Some((re, im)) = Self::solve_poly(coefficients) {
            // We only need real root between (0, 1) because we add endpoints according to param
            let roots = re
                .col_as_slice(0)
                .iter()
                .copied()
                .zip(im.col_as_slice(0).iter().copied())
                .filter(|(re, im)| 0.0 < *re && *re < 1.0 && *im == 0.0)
                .map(|(re, _)| re);

            let endpoints = if allow_endpoint {
                Some(0.0).into_iter().chain(Some(1.0))
            } else {
                None.into_iter().chain(None)
            };

            roots
                .chain(endpoints)
                .map(|t| Nearest::new_from_bezier(self, t, target))
                .min()
        } else if allow_endpoint {
            Some(Nearest::new_from_bezier(self, 0.0, target))
        } else {
            None
        }
    }

    pub fn split_at(&self, t: f64) -> (Self, Self) {
        let f = self.parametric_function();
        let p = f(t);

        let nt = 1.0 - t;
        let t2 = t * t;
        let _2tnt = 2.0 * t * nt;
        let nt2 = nt * nt;

        let left = Self::new(
            self.start,
            (
                t * self.ctrl1.0 + nt * self.start.0,
                t * self.ctrl1.1 + nt * self.start.1,
            ),
            (
                t2 * self.ctrl2.0 + _2tnt * self.ctrl1.0 + nt2 * self.start.0,
                t2 * self.ctrl2.1 + _2tnt * self.ctrl1.1 + nt2 * self.start.1,
            ),
            p,
        );

        let right = Self::new(
            p,
            (
                t2 * self.end.0 + _2tnt * self.ctrl2.0 + nt2 * self.ctrl1.0,
                t2 * self.end.1 + _2tnt * self.ctrl2.1 + nt2 * self.ctrl1.1,
            ),
            (
                t * self.end.0 + nt * self.ctrl2.0,
                t * self.end.1 + nt * self.ctrl2.1,
            ),
            self.end,
        );

        (left, right)
    }
}
