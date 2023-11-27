mod bezier;
mod segment;
mod point;
mod nearest;

pub use self::{
    bezier::Bezier,
    nearest::Nearest,
    point::{CornerPoint, CurvePoint, Point, PointExt, SmoothPoint},
    segment::Segment,
};

pub enum Curve {
    Segment(Segment),
    Bezier(Bezier),
}

impl Curve {
    pub fn new(start: &CurvePoint, end: &CurvePoint) -> Self {
        let sp = *start.point();
        let ep = *end.point();

        match (start.out_ctrl(), end.in_ctrl()) {
            (Some(ctrl1), Some(ctrl2)) => {
                Self::Bezier(Bezier::new(sp, ctrl1.into_owned(), ctrl2.into_owned(), ep))
            }
            (Some(ctrl), None) | (None, Some(ctrl)) => {
                Self::Bezier(Bezier::new_quad(sp, ctrl.into_owned(), ep))
            }
            (None, None) => Self::Segment(Segment::new(sp, ep)),
        }
    }

    pub fn at(&self, t: f64) -> Point {
        match self {
            Self::Bezier(b) => b.at(t),
            Self::Segment(l) => l.at(t),
        }
    }

    pub fn nearest_to(&self, target: &Point, allow_endpoint: bool) -> Option<Nearest> {
        match self {
            Self::Bezier(b) => b.nearest_to(target, allow_endpoint),
            Self::Segment(l) => l.nearest_to(target, allow_endpoint),
        }
    }
}
