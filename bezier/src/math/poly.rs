use alloc::vec::Vec;
use core::{iter::repeat, ops::Range};

use super::SturmSeq;

#[derive(Debug, Clone, PartialEq)]
pub struct Poly {
    c: Vec<f64>,
}

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

        self.c.iter().fold(0.0, |acc, c| acc * x + c)
    }

    pub fn real_roots(&self) -> Root {
        let c_abs_max = self
            .c
            .iter()
            .copied()
            .map(libm::fabs)
            .fold(0.0_f64, f64::max);

        let bound = libm::fabs(self.c[0] / c_abs_max);
        self.real_roots_in(-bound..bound)
    }

    pub fn real_roots_in(&self, range: Range<f64>) -> Root {
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
        let ranges = sturm.isolate_real_roots_iter(range, 1e-6);
        let roots: Vec<_> = ranges.filter_map(|r| self.newton_find_root_in(r)).collect();

        if roots.is_empty() {
            Root::None
        } else {
            Root::Roots(roots)
        }
    }

    pub fn newton_find_root_in(&self, range: Range<f64>) -> Option<f64> {
        todo!()
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
}
