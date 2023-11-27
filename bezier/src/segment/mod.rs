mod bezier;
mod line;
mod point;

use egui_plot::PlotPoint;

pub use self::{
    bezier::{Bezier, BezierOwned},
    line::LineSegment,
    point::{CornerPoint, CurvePoint, PlotPointExt, PointAction, SmoothPoint},
};

pub enum Segment<'a> {
    Line(LineSegment<'a>),
    Bezier(Bezier<'a>),
}

pub struct Nearest {
    pub t: f64,
    pub point: PlotPoint,
    pub distance: f64,
}

impl<'a> Segment<'a> {
    pub fn new(start: &'a CurvePoint, end: &'a CurvePoint) -> Self {
        let sp = start.point();
        let ep = end.point();

        match (start.out_ctrl(), end.in_ctrl()) {
            (Some(ctrl1), Some(ctrl2)) => Segment::Bezier(Bezier::new(sp, ctrl1, ctrl2, ep)),
            (Some(ctrl), None) | (None, Some(ctrl)) => {
                Segment::Bezier(Bezier::new_quad(sp, ctrl.as_ref(), ep))
            }
            (None, None) => Segment::Line(LineSegment::new(sp, ep)),
        }
    }

    pub fn nearest_to(&self, target: &PlotPoint) -> Option<Nearest> {
        match self {
            Self::Bezier(b) => b.nearest_to(target),
            Self::Line(l) => l.nearest_to(target),
        }
    }
}
