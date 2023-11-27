use std::borrow::Cow;

pub type Point = (f64, f64);

pub trait PointExt: Sized {
    fn from_x_y(x: f64, y: f64) -> Self;

    fn x(&self) -> f64;
    fn y(&self) -> f64;

    fn length_from_origin(&self) -> f64 {
        let (x, y) = (self.x(), self.y());
        (x * x + y * y).sqrt()
    }

    // theta between [0, 360]
    fn polar(&self) -> (f64, f64) {
        let (x, y) = (self.x(), self.y());

        if x == 0.0 && y == 0.0 {
            return (0.0, 0.0);
        }

        let r = self.length_from_origin();

        let mut theta = (x / self.length_from_origin()).acos().to_degrees();
        if y.is_sign_negative() {
            theta = 360.0 - theta;
        }

        (r, theta)
    }

    fn negative(&self) -> Self {
        Self::from_x_y(-self.x(), -self.y())
    }

    fn minus(&self, rhs: &Self) -> Self {
        Self::from_x_y(self.x() - rhs.x(), self.y() - rhs.y())
    }

    fn plus(&self, rhs: &Self) -> Self {
        Self::from_x_y(self.x() + rhs.x(), self.y() + rhs.y())
    }

    fn move_follow(&self, dir: f64, length: f64) -> Self {
        Self::from_x_y(
            self.x() + dir.to_radians().cos() * length,
            self.y() + dir.to_radians().sin() * length,
        )
    }
}

impl PointExt for Point {
    fn from_x_y(x: f64, y: f64) -> Self {
        (x, y)
    }

    fn x(&self) -> f64 {
        self.0
    }

    fn y(&self) -> f64 {
        self.1
    }
}

/// Control point is free, you can use
/// `{in/out}_ctrl_mut` / `update_{in/out}_ctrl` / `remove_{in/out}_ctrl` to change them as you
/// wish.
///
/// But center point can only be modified by `move_delta` and `move_to` function, which allows
/// the control points follow its movement.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CornerPoint {
    in_ctrl: Option<Point>,
    point: Point,
    out_ctrl: Option<Point>,
}

impl CornerPoint {
    pub fn new(point: Point) -> Self {
        Self {
            point,
            in_ctrl: None,
            out_ctrl: None,
        }
    }

    pub fn with_in_ctrl(mut self, point: Point) -> Self {
        self.in_ctrl.replace(point);
        self
    }

    pub fn with_out_ctrl(mut self, point: Point) -> Self {
        self.out_ctrl.replace(point);
        self
    }

    pub fn point(&self) -> &Point {
        &self.point
    }

    pub fn move_delta(&mut self, delta: Point, move_ctrls: bool) {
        self.point = self.point.plus(&delta);
        if move_ctrls {
            if let Some(m) = self.in_ctrl_mut() {
                *m = m.plus(&delta)
            }
            if let Some(m) = self.out_ctrl_mut() {
                *m = m.plus(&delta)
            }
        }
    }

    pub fn move_to(&mut self, target: Point, move_ctrls: bool) {
        let delta = target.minus(&self.point);
        self.move_delta(delta, move_ctrls);
    }

    pub fn has_in_ctrl(&self) -> bool {
        self.in_ctrl.is_some()
    }

    pub fn in_ctrl(&self) -> Option<&Point> {
        self.in_ctrl.as_ref()
    }

    pub fn in_ctrl_mut(&mut self) -> Option<&mut Point> {
        self.in_ctrl.as_mut()
    }

    pub fn update_in_ctrl(&mut self, val: Point) {
        self.in_ctrl.replace(val);
    }

    pub fn remove_in_ctrl(&mut self) {
        self.in_ctrl.take();
    }

    pub fn has_out_ctrl(&self) -> bool {
        self.out_ctrl.is_some()
    }

    pub fn out_ctrl(&self) -> Option<&Point> {
        self.out_ctrl.as_ref()
    }

    pub fn out_ctrl_mut(&mut self) -> Option<&mut Point> {
        self.out_ctrl.as_mut()
    }

    pub fn update_out_ctrl(&mut self, val: Point) {
        self.out_ctrl.replace(val);
    }

    pub fn remove_out_ctrl(&mut self) {
        self.out_ctrl.take();
    }
}

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
pub struct SmoothPoint {
    point: Point,
    theta: f64,
    in_length: f64,
    out_length: f64,
}

impl SmoothPoint {
    pub fn new(point: Point, theta: f64, in_length: f64, out_length: f64) -> Self {
        Self::new_unchecked(point, theta % (360.0), in_length.abs(), out_length.abs())
    }

    fn new_unchecked(point: Point, rad: f64, in_length: f64, out_length: f64) -> Self {
        Self {
            point,
            theta: rad,
            in_length,
            out_length,
        }
    }

    pub fn horizontal(point: Point, in_length: f64, out_length: f64) -> Self {
        Self::new_unchecked(point, 0.0, in_length.abs(), out_length.abs())
    }

    pub fn vertical(point: Point, in_length: f64, out_length: f64) -> Self {
        Self::new_unchecked(point, 90.0, in_length.abs(), out_length.abs())
    }

    pub fn point(&self) -> &Point {
        &self.point
    }

    pub fn point_mut(&mut self) -> &mut Point {
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

    pub fn in_ctrl(&self) -> Point {
        self.point.move_follow(self.theta + 180.0, self.in_length)
    }

    pub fn out_ctrl(&self) -> Point {
        self.point.move_follow(self.theta, self.out_length)
    }

    pub fn update_theta(&mut self, theta: f64) {
        self.theta = theta % 360.0;
    }

    pub fn update_in_length(&mut self, val: f64) {
        self.in_length = val.abs();
    }

    pub fn update_out_length(&mut self, val: f64) {
        self.out_length = val.abs();
    }

    pub fn move_in_ctrl_to(&mut self, val: &Point) {
        let v = self.point.minus(val);
        (self.in_length, self.theta) = v.polar();
    }

    pub fn move_out_ctrl_to(&mut self, val: &Point) {
        let v = val.minus(&self.point);
        (self.out_length, self.theta) = v.polar();
    }
}

#[derive(Clone)]
pub enum CurvePoint {
    Corner(CornerPoint),
    Smooth(SmoothPoint),
}

impl CurvePoint {
    pub fn point(&self) -> &Point {
        match self {
            Self::Corner(c) => c.point(),
            Self::Smooth(s) => s.point(),
        }
    }

    pub fn in_ctrl(&self) -> Option<Cow<'_, Point>> {
        match self {
            Self::Corner(c) => c.in_ctrl.as_ref().map(Cow::Borrowed),
            Self::Smooth(s) => Some(Cow::Owned(s.in_ctrl())),
        }
    }

    pub fn out_ctrl(&self) -> Option<Cow<'_, Point>> {
        match self {
            Self::Corner(c) => c.out_ctrl.as_ref().map(Cow::Borrowed),
            Self::Smooth(s) => Some(Cow::Owned(s.out_ctrl())),
        }
    }

    pub fn update_in_ctrl(&mut self, val: Point) {
        match self {
            Self::Corner(c) => c.update_in_ctrl(val),
            Self::Smooth(s) => s.move_in_ctrl_to(&val),
        };
    }

    pub fn update_out_ctrl(&mut self, val: Point) {
        match self {
            Self::Corner(c) => c.update_out_ctrl(val),
            Self::Smooth(s) => s.move_out_ctrl_to(&val),
        };
    }
}

impl From<CornerPoint> for CurvePoint {
    fn from(value: CornerPoint) -> Self {
        Self::Corner(value)
    }
}

impl From<SmoothPoint> for CurvePoint {
    fn from(value: SmoothPoint) -> Self {
        Self::Smooth(value)
    }
}
