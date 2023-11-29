use alloc::vec;

use dyn_stack::PodStack;
use faer_core::{Mat, Parallelism};
use faer_evd::{ComputeVectors, EvdParams};

use super::Nearest;
use crate::Point2D;

pub struct Bezier<P> {
    pub start: P,
    pub ctrl1: P,
    pub ctrl2: P,
    pub end: P,
}

impl<P> Bezier<P> {
    pub fn new(start: P, ctrl1: P, ctrl2: P, end: P) -> Self {
        Self {
            start,
            end,
            ctrl1,
            ctrl2,
        }
    }
}

impl<P: Point2D> Bezier<P> {
    fn quad_to_cubic(a: &P, b: &P) -> P {
        b.minus(a).scale(2.0 / 3.0).plus(a)
    }

    pub fn new_quad(start: P, ctrl: P, end: P) -> Self {
        let ctrl1 = Self::quad_to_cubic(&start, &ctrl);
        let ctrl2 = Self::quad_to_cubic(&end, &ctrl);

        Self::new(start, ctrl1, ctrl2, end)
    }

    fn parametric_function_coefficients(&self) -> [P; 4] {
        [
            self.start
                .negative()
                .plus(&self.ctrl1.minus(&self.ctrl2).scale(3.0))
                .plus(&self.end),
            self.start
                .minus(&self.ctrl1.scale(2.0))
                .plus(&self.ctrl2)
                .scale(3.0),
            self.start.minus(&self.ctrl1).scale(-3.0),
            self.start.clone(),
        ]
    }

    pub fn parametric_function(&self) -> impl Fn(f64) -> P {
        let [a, b, c, d] = self.parametric_function_coefficients();

        move |t| {
            let t2 = t * t;
            let t3 = t2 * t;

            a.scale(t3).plus(&b.scale(t2)).plus(&c.scale(t)).plus(&d)
        }
    }

    pub fn at(&self, t: f64) -> P {
        assert!((0.0..=1.0).contains(&t));
        self.parametric_function()(t)
    }

    fn distance_derivative_coefficients(&self, target: &P) -> [f64; 6] {
        let [a, b, c, d] = self.parametric_function_coefficients();

        let dt = d.minus(target);

        [
            3.0 * a.dot(&a),
            5.0 * a.dot(&b),
            4.0 * a.dot(&c) + 2.0 * b.dot(&b),
            3.0 * a.dot(&dt) + 3.0 * b.dot(&c),
            2.0 * b.dot(&dt) + c.dot(&c),
            c.dot(&dt),
        ]
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
}

impl<P: Point2D> Bezier<P> {
    pub fn split_at(&self, t: f64) -> (Self, Self) {
        let p = self.at(t);

        let nt = 1.0 - t;
        let t2 = t * t;
        let _2tnt = 2.0 * t * nt;
        let nt2 = nt * nt;

        let left = Self::new(
            self.start.clone(),
            self.ctrl1.scale(t).plus(&self.start.scale(nt)),
            self.ctrl2
                .scale(t2)
                .plus(&self.ctrl1.scale(_2tnt))
                .plus(&self.start.scale(nt2)),
            p.clone(),
        );

        let right = Self::new(
            p,
            self.end
                .scale(t2)
                .plus(&self.ctrl2.scale(_2tnt))
                .plus(&self.ctrl1.scale(nt2)),
            self.end.scale(t).plus(&self.ctrl2.scale(nt)),
            self.end.clone(),
        );

        (left, right)
    }

    /// Calculate the nearest point on the segment to a provided target point.
    pub fn nearest_to(&self, target: &P, allow_endpoint: bool) -> Option<Nearest<P>> {
        let coefficients = self.distance_derivative_coefficients(target);

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
}
