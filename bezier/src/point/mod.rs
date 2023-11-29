mod corner;
mod smooth;

use alloc::borrow::Cow;
use core::cmp::Ordering;

pub use self::{corner::*, smooth::*};

pub type TuplePoint2D = (f64, f64);
pub type ArrayPoint2D = [f64; 2];

pub trait Point2D: Clone {
    fn x(&self) -> f64;
    fn y(&self) -> f64;

    fn from_xy(x: f64, y: f64) -> Self;

    #[inline(always)]
    fn tuple(&self) -> TuplePoint2D {
        (self.x(), self.y())
    }

    #[inline(always)]
    fn array(&self) -> ArrayPoint2D {
        [self.x(), self.y()]
    }

    #[inline(always)]
    fn length_from_origin(&self) -> f64 {
        let [x, y] = self.array();
        libm::sqrt(x * x + y * y)
    }

    #[inline(always)]
    fn distance(&self, rhs: &Self) -> f64 {
        self.minus(rhs).length_from_origin()
    }

    // theta, r;
    // theta is degree, between [0, 360]
    fn polar(&self) -> (f64, f64) {
        let [x, y] = (*self).array();

        let r = self.length_from_origin();

        if r == 0.0 {
            return (0.0, 0.0);
        }

        let mut theta = libm::acos(x / r).to_degrees();
        if y.is_sign_negative() {
            theta = 360.0 - theta;
        }

        (r, theta)
    }

    #[inline(always)]
    fn negative(&self) -> Self {
        let [x, y] = self.array();
        Self::from_xy(-x, -y)
    }

    #[inline(always)]
    fn plus(&self, rhs: &Self) -> Self {
        let [x1, y1] = self.array();
        let [x2, y2] = rhs.array();
        Self::from_xy(x1 + x2, y1 + y2)
    }

    #[inline(always)]
    fn minus(&self, rhs: &Self) -> Self {
        let [x1, y1] = self.array();
        let [x2, y2] = rhs.array();
        Self::from_xy(x1 - x2, y1 - y2)
    }

    #[inline(always)]
    fn multiply(&self, rhs: &Self) -> Self {
        let [x1, y1] = self.array();
        let [x2, y2] = rhs.array();
        Self::from_xy(x1 * x2, y1 * y2)
    }

    #[inline(always)]
    fn dot(&self, rhs: &Self) -> f64 {
        let [x1, y1] = self.array();
        let [x2, y2] = rhs.array();
        x1 * x2 + y1 * y2
    }

    #[inline(always)]
    fn scale(&self, rhs: f64) -> Self {
        let [x, y] = self.array();
        Self::from_xy(x * rhs, y * rhs)
    }

    #[inline(always)]
    fn move_follow(&self, dir: f64, length: f64) -> Self {
        let delta = Self::from_xy(
            libm::cos(dir.to_radians()) * length,
            libm::sin(dir.to_radians()) * length,
        );
        self.plus(&delta)
    }

    #[inline(always)]
    fn total_cmp(&self, other: &Self) -> Ordering {
        let [x, y] = self.array();
        let [x2, y2] = other.array();

        match x.total_cmp(&x2) {
            Ordering::Equal => {}
            ord => return ord,
        }

        y.total_cmp(&y2)
    }
}

impl Point2D for TuplePoint2D {
    #[inline(always)]
    fn x(&self) -> f64 {
        self.0
    }

    #[inline(always)]
    fn y(&self) -> f64 {
        self.1
    }

    #[inline(always)]
    fn from_xy(x: f64, y: f64) -> Self {
        (x, y)
    }
}

impl Point2D for ArrayPoint2D {
    #[inline(always)]
    fn x(&self) -> f64 {
        self[0]
    }

    #[inline(always)]
    fn y(&self) -> f64 {
        self[1]
    }

    #[inline(always)]
    fn from_xy(x: f64, y: f64) -> Self {
        [x, y]
    }
}

#[derive(Clone)]
pub enum CurvePoint<P> {
    Corner(CornerPoint<P>),
    Smooth(SmoothPoint<P>),
}

impl<P> CurvePoint<P> {
    pub fn point(&self) -> &P {
        match self {
            Self::Corner(c) => c.point(),
            Self::Smooth(s) => s.point(),
        }
    }
}

impl<P: Point2D> CurvePoint<P> {
    pub fn in_ctrl(&self) -> Option<Cow<'_, P>> {
        match self {
            Self::Corner(c) => c.in_ctrl().map(Cow::Borrowed),
            Self::Smooth(s) => Some(Cow::Owned(s.in_ctrl())),
        }
    }

    pub fn out_ctrl(&self) -> Option<Cow<'_, P>> {
        match self {
            Self::Corner(c) => c.out_ctrl().map(Cow::Borrowed),
            Self::Smooth(s) => Some(Cow::Owned(s.out_ctrl())),
        }
    }

    pub fn update_in_ctrl(&mut self, val: P) {
        match self {
            Self::Corner(c) => c.update_in_ctrl(val),
            Self::Smooth(s) => s.move_in_ctrl_to(&val),
        };
    }

    pub fn update_out_ctrl(&mut self, val: P) {
        match self {
            Self::Corner(c) => c.update_out_ctrl(val),
            Self::Smooth(s) => s.move_out_ctrl_to(&val),
        };
    }
}

impl<P> From<CornerPoint<P>> for CurvePoint<P> {
    fn from(value: CornerPoint<P>) -> Self {
        Self::Corner(value)
    }
}

impl<P> From<SmoothPoint<P>> for CurvePoint<P> {
    fn from(value: SmoothPoint<P>) -> Self {
        Self::Smooth(value)
    }
}
