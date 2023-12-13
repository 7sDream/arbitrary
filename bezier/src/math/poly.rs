use alloc::vec::Vec;
use core::{
    iter::repeat,
    ops::{Range, RangeInclusive},
};

use super::SturmSeq;

#[derive(Debug, Clone, PartialEq)]
pub struct Poly {
    c: Vec<f64>,
}

#[derive(Debug)]
pub enum Root {
    None,
    Any,
    Roots(Vec<f64>),
}

impl Poly {
    #[inline(always)]
    pub fn zero() -> Self {
        Self { c: vec![0.0] }
    }

    #[inline(always)]
    pub fn degree(&self) -> usize {
        self.c.len() - 1
    }

    #[inline(always)]
    pub fn is_zero(&self) -> bool {
        *self.c.first().unwrap() == 0.0
    }

    #[inline(always)]
    pub fn coefficients(&self) -> &'_ [f64] {
        &self.c
    }

    pub fn neg(&self) -> Poly {
        self.c.iter().map(|c| -c).collect()
    }

    pub fn derivative(&self) -> Poly {
        if self.degree() == 0 {
            return Self::zero();
        }

        let degree = self.degree();
        self.coefficients()
            .iter()
            .enumerate()
            .take(degree)
            .map(|(i, c)| (degree - i) as f64 * c)
            .collect()
    }

    fn div_once(dividend: &mut Poly, divisor: &Poly) -> Option<f64> {
        if dividend.is_zero() || dividend.degree() < divisor.degree() {
            return None;
        }

        let q = dividend.c[0] / divisor.c[0];

        let a = dividend.c.iter().skip(1).copied();
        let b = divisor.c.iter().copied().chain(repeat(0.0)).skip(1);
        let r = a.zip(b).map(|(a, b)| a - b * q).collect();

        *dividend = r;
        Some(q)
    }

    pub fn div(&self, divisor: &Poly) -> (Poly, Poly) {
        if self.degree() < divisor.degree() {
            return (Self::zero(), self.clone());
        }

        let q_len = self.degree() - divisor.degree() + 1;

        let mut r = self.clone();
        let mut q: Poly = core::iter::from_fn(|| Self::div_once(&mut r, divisor)).collect();

        q.c.resize(q_len, 0.0);

        (q, r)
    }

    pub fn eval(&self, x: f64) -> f64 {
        assert!(!x.is_nan());

        if self.degree() > 0 && x.is_infinite() {
            return f64::INFINITY
                * if self.c[0].is_sign_negative() {
                    -1.0
                } else {
                    1.0
                }
                * if x.is_sign_positive() || self.degree() % 2 == 0 {
                    1.0
                } else {
                    -1.0
                };
        }

        // mul_add is slower then acc * x + c, but more accurate according to doc
        self.c.iter().copied().fold(0.0, |acc, c| acc.mul_add(x, c))
    }

    pub fn real_roots(&self) -> Root {
        let c_abs_max = self
            .c
            .iter()
            .copied()
            .map(libm::fabs)
            .fold(0.0_f64, f64::max);

        let bound = 1.0 + c_abs_max / libm::fabs(self.c[0]);
        self.real_roots_in(-bound..=bound)
    }

    pub fn real_roots_in(&self, range: RangeInclusive<f64>) -> Root {
        if self.degree() == 0 {
            if self.is_zero() {
                return Root::Any;
            } else {
                return Root::None;
            }
        }

        if self.degree() == 1 {
            let [a, b] = [self.c[0], self.c[1]];
            let root = -b / a;
            if range.contains(&root) {
                return Root::Roots(vec![root]);
            } else {
                return Root::None;
            }
        }

        let sturm = SturmSeq::new(self);
        let ranges = sturm.isolate_real_roots_iter(*range.start(), *range.end(), f64::MAX);
        let mut roots: Vec<_> = ranges
            .filter_map(|(start, end)| self.newton_find_root_in(&sturm[1], start, end))
            .collect();

        if self.eval(*range.start()) == 0.0 {
            roots.push(*range.start())
        }

        if roots.is_empty() {
            Root::None
        } else {
            Root::Roots(roots)
        }
    }

    // Get a series point in the [start, end] range, in binary search order:
    //
    // e.g. for [0.0, 1.0]:
    //
    //                            1/2
    //              1/4                          3/4
    //      1/8            3/8           5/8             7/8
    // 1/16     3/16   5/16   7/16   9/16   11/16   13/16   15/16
    //   ....................................................
    //
    // The order is row by row, bigger first.
    //
    // The iterator stops when reach the row which will split the region more then `max` pieces.
    // `max` must be a power of two.
    fn interval_binary_split_points(
        start: f64, end: f64, mut max: usize,
    ) -> impl Iterator<Item = f64> {
        assert!(max.is_power_of_two());

        let mut acc = end;
        let mut step = end - start;

        core::iter::from_fn(move || {
            acc -= step;
            if acc <= start {
                step /= 2.0;
                max /= 2;
                if max == 0 {
                    return None;
                }
                acc = end - step;
            }

            Some(acc)
        })
    }

    // left open right close interval (start, end]
    fn newton_find_root_in(&self, d: &Poly, start: f64, end: f64) -> Option<f64> {
        [end, start]
            .into_iter()
            .chain(Self::interval_binary_split_points(
                start,
                end,
                1_usize.rotate_right(1),
            ))
            .filter_map(|at| self.newton_find_root_at(d, at))
            .find(|x| start < *x && *x <= end)
    }

    fn newton_find_root_at(&self, d: &Poly, at: f64) -> Option<f64> {
        let eps = -f64::EPSILON..=f64::EPSILON;
        let mut x = at;

        loop {
            let dv = d.eval(x);
            let fv = self.eval(x);

            if eps.contains(&fv) {
                return Some(x);
            }

            // meet local extremum, but not zero
            if eps.contains(&dv) {
                return None;
            }

            let delta = fv / dv;
            x -= delta;

            if eps.contains(&delta) {
                return Some(x);
            }
        }
    }
}

impl FromIterator<f64> for Poly {
    fn from_iter<T: IntoIterator<Item = f64>>(iter: T) -> Self {
        let coefficients: Vec<f64> = iter
            .into_iter()
            .skip_while(|c| *c == 0.0)
            .inspect(|c| {
                assert!(c.is_finite());
            })
            .collect();
        if coefficients.is_empty() {
            Self::zero()
        } else {
            Self { c: coefficients }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn poly_zero() {
        assert_eq!([].into_iter().collect::<Poly>(), Poly::zero());
        assert_eq!([0.0].into_iter().collect::<Poly>(), Poly::zero());
        assert_eq!([0.0, 0.0].into_iter().collect::<Poly>(), Poly::zero());
    }

    #[test]
    fn high_degree_poly_derivative() {
        // 5x^5 + 4x^4 + x^3 + 2x^2 + 3x + 1
        let p: Poly = [5.0, 4.0, 1.0, 2.0, 3.0, 1.0].into_iter().collect();
        let d = p.derivative();
        // 25x^4 + 16x^3 + 3x^2 + 4x + 3
        assert_eq!(d.c, vec![25.0, 16.0, 3.0, 4.0, 3.0])
    }

    #[test]
    fn low_degree_derivative() {
        // 3x+1
        let p: Poly = [3.0, 1.0].into_iter().collect();
        let d = p.derivative();
        // 3
        assert_eq!(d.c, vec![3.0])
    }

    #[test]
    fn const_derivative() {
        let p: Poly = [1.0].into_iter().collect();
        assert_eq!(p.derivative(), Poly::zero());
        assert_eq!(Poly::zero().derivative(), Poly::zero())
    }

    #[test]
    fn poly_div() {
        let dividend: Poly = [1.0, -12.0, 0.0, -42.0].into_iter().collect();
        let divisor: Poly = [1.0, -3.0].into_iter().collect();
        let (q, r) = dividend.div(&divisor);
        assert_eq!(q.c, [1.0, -9.0, -27.0]);
        assert_eq!(r.c, [-123.0]);
    }

    #[test]
    fn poly_div_no_reminder() {
        let dividend: Poly = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0].into_iter().collect();
        let divisor: Poly = [1.0, 0.0, 0.0, 0.0].into_iter().collect();
        let (q, r) = dividend.div(&divisor);
        assert_eq!(q.c, [1.0, 0.0, 0.0]);
        assert!(r.is_zero());
    }

    #[test]
    fn poly_eval() {
        let poly: Poly = [2.0, -6.0, 2.0, -1.0].into_iter().collect();
        assert_eq!(poly.eval(3.0), 5.0);
    }

    #[test]
    fn poly_eval_inf() {
        let poly: Poly = [2.0, -6.0, 2.0, -1.0].into_iter().collect();
        assert_eq!(poly.eval(f64::INFINITY), f64::INFINITY);
        assert_eq!(poly.eval(f64::NEG_INFINITY), f64::NEG_INFINITY);
    }

    #[test]
    fn poly_real_roots() {
        let poly: Poly = [1.0, -2.0, 0.25, 0.75].into_iter().collect();
        dbg!(poly.real_roots());
    }
}
