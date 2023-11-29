use super::{Nearest, Point};

pub struct Segment {
    start: Point,
    end: Point,
}

impl Segment {
    pub fn new(start: Point, end: Point) -> Self {
        Self { start, end }
    }

    fn parametric_function_coefficients(&self) -> [Point; 2] {
        [
            (self.end.0 - self.start.0, self.end.1 - self.start.1),
            self.start,
        ]
    }

    pub fn parametric_function(&self) -> impl Fn(f64) -> (f64, f64) {
        let [a, b] = self.parametric_function_coefficients();

        move |t| {
            let x = a.0 * t + b.0;
            let y = a.1 * t + b.1;
            (x, y)
        }
    }

    pub fn at(&self, t: f64) -> Point {
        self.parametric_function()(t)
    }

    fn distance_derivative_coefficients(&self, target: &Point) -> [f64; 2] {
        let [a, b] = self.parametric_function_coefficients();

        let ax = a.0 * a.0;
        let ay = a.1 * a.1;

        let bx = (b.0 - target.0) * a.0;
        let by = (b.1 - target.1) * a.1;

        [ax + ay, bx + by]
    }

    pub fn nearest_to(&self, target: &Point, allow_endpoint: bool) -> Option<Nearest> {
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
