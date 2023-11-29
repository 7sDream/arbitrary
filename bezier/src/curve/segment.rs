use super::{Nearest, Point2D};

pub struct Segment<P> {
    start: P,
    end: P,
}

impl<P> Segment<P> {
    pub fn new(start: P, end: P) -> Self {
        Self { start, end }
    }
}

impl<P: Point2D> Segment<P> {
    fn parametric_function_coefficients(&self) -> [P; 2] {
        [self.end.minus(&self.start), self.start.clone()]
    }

    pub fn parametric_function(&self) -> impl Fn(f64) -> P {
        let [a, b] = self.parametric_function_coefficients();

        move |t| a.scale(t).plus(&b)
    }

    pub fn at(&self, t: f64) -> P {
        assert!((0.0..=1.0).contains(&t));
        self.parametric_function()(t)
    }

    fn distance_derivative_coefficients(&self, target: &P) -> [f64; 2] {
        let [a, b] = self.parametric_function_coefficients();

        [a.dot(&a), (b.minus(target)).dot(&a)]

        // let ax = a.x() * a.x();
        // let ay = a.y() * a.y();

        // let bx = (b.x() - target.x()) * a.x();
        // let by = (b.y() - target.y()) * a.y();

        // [ax + ay, bx + by]
    }

    pub fn nearest_to(&self, target: &P, allow_endpoint: bool) -> Option<Nearest<P>> {
        let [a, b] = self.distance_derivative_coefficients(target);

        let t = if a == 0.0 { 0.0 } else { -b / a };

        if !allow_endpoint && t == 0.0 || t == 1.0 {
            return None;
        }

        if (0.0..=1.0).contains(&t) {
            return Some(Nearest::new_from_segment(self, t, target));
        }

        None
    }
}
