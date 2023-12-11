use alloc::vec::Vec;
use core::{
    cmp::Ordering,
    iter::repeat,
    num::FpCategory,
    ops::{Deref, Range, RangeBounds, RangeInclusive},
};

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
    pub fn zero() -> Self {
        Self { c: vec![0.0] }
    }

    pub fn degree(&self) -> usize {
        self.c.len() - 1
    }

    pub fn is_zero(&self) -> bool {
        *self.c.first().unwrap() == 0.0
    }

    pub fn neg(&self) -> Poly {
        self.c.iter().map(|c| -c).collect()
    }

    pub fn derivative(&self) -> Poly {
        if self.c.len() == 1 {
            return Self { c: vec![0.0] };
        }

        let degree = self.c.len() - 1;
        self.c
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
        if self.degree() > 0 && x.is_infinite() {
            return f64::INFINITY
                * self.c[0].signum()
                * if x.is_sign_positive() || self.degree() % 2 == 0 {
                    1.0
                } else {
                    -1.0
                };
        }

        let mut i = self.c.iter();
        let an = i.next().unwrap();
        i.fold(*an, |acc, c| acc * x + c)
    }

    fn sturm_seq(&self) -> SturmSeq {
        let mut result = Vec::with_capacity(self.degree());

        result.push(self.clone());

        if self.degree() < 1 {
            return SturmSeq(result);
        }

        let mut divided = self.clone();
        let mut last = self.derivative();
        loop {
            let (_, r) = divided.div(&last);
            if r.is_zero() {
                break;
            } else {
                result.push(last.clone());
                divided = last;
                last = r.neg();
            }
        }

        result.push(last);

        SturmSeq(result)
    }

    pub fn real_roots(&self) -> Root {
        let max = self.c.iter().copied().fold(f64::INFINITY, |acc, c| {
            if acc.total_cmp(&c) == Ordering::Less {
                c
            } else {
                acc
            }
        });

        let normalized = self.c[0] / max;
        self.real_roots_in(-normalized..normalized)
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

        let sturm = self.sturm_seq();
        let state = IsolateRootState::default();
        let root_ranges = sturm.isolate_roots_iter(range, state);

        let roots: Vec<_> = root_ranges
            .filter_map(|r| self.newton_find_root_in(r))
            .collect();

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

#[derive(Debug, Clone, PartialEq)]
struct SturmSeq(Vec<Poly>);

impl Deref for SturmSeq {
    type Target = [Poly];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Sign {
    Zero,
    Positive,
    Negative,
}

#[derive(Default)]
struct IsolateRootState {
    queue: Vec<(f64, Option<usize>, f64, Option<usize>)>,
}

impl SturmSeq {
    fn eval(&self, x: f64) -> impl Iterator<Item = f64> + '_ {
        self.0.iter().map(move |p| p.eval(x))
    }

    fn signs_at(&self, x: f64) -> impl Iterator<Item = Sign> + '_ {
        self.eval(x).map(|x| {
            if x == 0.0 {
                Sign::Zero
            } else if x.is_sign_negative() {
                Sign::Negative
            } else {
                Sign::Positive
            }
        })
    }

    pub fn sign_changes_at(&self, x: f64) -> usize {
        let mut i = self.signs_at(x);
        let first = i.next().unwrap();
        i.fold((first, 0), |(last, acc), current| {
            if current == Sign::Zero || last == current {
                (last, acc)
            } else {
                (current, acc + 1)
            }
        })
        .1
    }

    pub fn isolate_roots_iter(
        &self, range: Range<f64>, mut state: IsolateRootState,
    ) -> impl Iterator<Item = Range<f64>> + '_ {
        state.queue.push((range.start, None, range.end, None));
        core::iter::from_fn(move || self.isolate_roots_iter_fn(&mut state))
    }

    fn isolate_roots_iter_fn<'i, 'x>(
        &'i self, state: &'x mut IsolateRootState,
    ) -> Option<Range<f64>> {
        while let Some((start, d1, end, d2)) = state.queue.pop() {
            let d1 = d1.unwrap_or_else(|| self.sign_changes_at(start));
            let d2 = d2.unwrap_or_else(|| self.sign_changes_at(end));

            if d1 == d2 {
                continue;
            }

            if d2 + 1 == d1 {
                return Some(start..end);
            }

            let mid = (start + end) / 2.0;
            state.queue.push((start, Some(d1), end, None));
            state.queue.push((mid, None, end, Some(d2)));
        }

        None
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
    fn poly_sturm_seq() {
        let poly: Poly = [1.0, 1.0, 0.0, -1.0, -1.0].into_iter().collect();
        let strum = poly.sturm_seq();

        assert_eq!(strum.len(), 5);
        assert_eq!(strum[0], poly);
        assert_eq!(strum[1], poly.derivative());
        assert_eq!(strum[2].c, [3.0 / 16.0, 3.0 / 4.0, 15.0 / 16.0]);
        assert_eq!(strum[3].c, [-32.0, -64.0]);
        assert_eq!(strum[4].c, [-3.0 / 16.0]);

        assert_eq!(strum.sign_changes_at(f64::NEG_INFINITY), 3);
        assert_eq!(strum.sign_changes_at(f64::INFINITY), 1);
    }
}
