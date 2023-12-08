use alloc::vec::Vec;
use core::iter::repeat;

#[derive(Debug, Clone, PartialEq)]
pub struct Poly {
    c: Vec<f64>,
}

impl Poly {
    pub fn zero() -> Self {
        Self { c: vec![0.0] }
    }

    pub fn degree(&self) -> usize {
        self.c.len() - 1
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

    fn div_once(dividend: &Poly, divisor: &Poly) -> (f64, Poly) {
        let quotient = dividend.c[0] / divisor.c[0];

        let a = dividend.c.iter().skip(1).copied();
        let b = divisor.c.iter().copied().chain(repeat(0.0)).skip(1);
        let reminder = a.zip(b).map(|(a, b)| a - b * quotient).collect();

        (quotient, reminder)
    }

    pub fn div(&self, divisor: &Poly) -> (Poly, Poly) {
        if self.degree() < divisor.degree() {
            return (Self::zero(), self.clone());
        }

        let quotient_coefficient_len = self.degree() - divisor.degree() + 1;

        let mut remainder = self.clone();
        let mut quotient: Poly = core::iter::from_fn(|| {
            if remainder.degree() >= divisor.degree() {
                let (q, r) = Self::div_once(&remainder, divisor);
                remainder = r;
                Some(q)
            } else {
                None
            }
        })
        .collect();

        quotient.c.resize(quotient_coefficient_len, 0.0);

        (quotient, remainder)
    }
}

impl FromIterator<f64> for Poly {
    fn from_iter<T: IntoIterator<Item = f64>>(iter: T) -> Self {
        let coefficients: Vec<f64> = iter.into_iter().skip_while(|c| *c == 0.0).collect();
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
        // 3x+1
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
}
