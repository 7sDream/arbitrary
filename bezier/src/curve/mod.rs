mod bezier;
mod segment;
mod nearest;

pub use self::{bezier::Bezier, nearest::Nearest, segment::Segment};
use crate::{CurvePoint, Point2D};

pub enum Curve<P> {
    Segment(Segment<P>),
    Bezier(Bezier<P>),
}

impl<P: Point2D> Curve<P> {
    pub fn new(start: &CurvePoint<P>, end: &CurvePoint<P>) -> Self {
        let sp = start.point().clone();
        let ep = end.point().clone();

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

    pub fn at(&self, t: f64) -> P {
        match self {
            Self::Bezier(b) => b.at(t),
            Self::Segment(l) => l.at(t),
        }
    }

    pub fn nearest_to(&self, target: &P, allow_endpoint: bool) -> Option<Nearest<P>> {
        match self {
            Self::Bezier(b) => b.nearest_to(target, allow_endpoint),
            Self::Segment(l) => l.nearest_to(target, allow_endpoint),
        }
    }
}
