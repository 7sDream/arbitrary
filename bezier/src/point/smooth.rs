use super::Point2D;

/// SmoothPoint keeps the point and two ctrl point collinear.
///
/// Center point is free, you can move it using `point_mut` function.
///
/// `theta`` is the degree of angel between out ctrl handler and X axis, in [0, 360),
/// you can change it using `update_theta`.
///
/// `in/out_length` is handler length of two ctrl point,
///
/// The position of ctrl points is calculated by parameters above, so
/// change them will changes position of two ctrl point automatically.
///
/// You can also directly update position of ctrl, it will changes another ctrl
/// point position to keep the collinear property.
#[derive(Clone)]
pub struct SmoothPoint<P> {
    point: P,
    theta: f64,
    in_length: f64,
    out_length: f64,
}

/// New
impl<P> SmoothPoint<P> {
    pub fn new(point: P, theta: f64, in_length: f64, out_length: f64) -> Self {
        Self::new_unchecked(
            point,
            theta % (360.0),
            libm::fabs(in_length),
            libm::fabs(out_length),
        )
    }

    fn new_unchecked(point: P, rad: f64, in_length: f64, out_length: f64) -> Self {
        Self {
            point,
            theta: rad,
            in_length,
            out_length,
        }
    }

    pub fn horizontal(point: P, in_length: f64, out_length: f64) -> Self {
        Self::new_unchecked(point, 0.0, libm::fabs(in_length), libm::fabs(out_length))
    }

    pub fn vertical(point: P, in_length: f64, out_length: f64) -> Self {
        Self::new_unchecked(point, 90.0, libm::fabs(in_length), libm::fabs(out_length))
    }
}

/// Getter/setter
impl<P> SmoothPoint<P> {
    pub fn point(&self) -> &P {
        &self.point
    }

    pub fn point_mut(&mut self) -> &mut P {
        &mut self.point
    }

    pub fn theta(&self) -> f64 {
        self.theta
    }

    pub fn in_length(&self) -> f64 {
        self.in_length
    }

    pub fn out_length(&self) -> f64 {
        self.out_length
    }

    pub fn update_theta(&mut self, theta: f64) {
        self.theta = theta % 360.0;
    }

    pub fn update_in_length(&mut self, val: f64) {
        self.in_length = libm::fabs(val);
    }

    pub fn update_out_length(&mut self, val: f64) {
        self.out_length = libm::fabs(val);
    }
}

/// Calculated
impl<P: Point2D> SmoothPoint<P> {
    pub fn in_ctrl(&self) -> P {
        self.point.move_follow(self.theta + 180.0, self.in_length)
    }

    pub fn out_ctrl(&self) -> P {
        self.point.move_follow(self.theta, self.out_length)
    }
}

/// Move
impl<P: Point2D> SmoothPoint<P> {
    pub fn move_in_ctrl_to(&mut self, val: &P) {
        let v = self.point.minus(val);
        (self.in_length, self.theta) = v.polar();
    }

    pub fn move_out_ctrl_to(&mut self, val: &P) {
        let v = val.minus(&self.point);
        (self.out_length, self.theta) = v.polar();
    }
}
