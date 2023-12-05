use bezier::{CornerPoint, CurvePoint, Nearest, Point2D, Shape, SmoothPoint};
use eframe::{
    egui::{Id, Ui},
    epaint::Pos2,
};
use egui_plot::{PlotResponse, PlotTransform};

use self::{corner::CornerPointInteract, smooth::SmoothPointInteract};
use crate::point::Point;

mod point;
mod corner;
mod smooth;

enum PointAction {
    Click,
    Delete,
    ConvertToCorner,
    ConvertToSmooth,
}

struct CurvePointInteract<'a>(&'a mut CurvePoint<Point>);

impl<'a> CurvePointInteract<'a> {
    pub fn interact(
        &mut self, ui: &mut Ui, id: Id, transform: &PlotTransform,
    ) -> Option<PointAction> {
        match self.0 {
            CurvePoint::Corner(cp) => CornerPointInteract::new(cp).interact(ui, id, transform),
            CurvePoint::Smooth(sp) => SmoothPointInteract::new(sp).interact(ui, id, transform),
        }
    }
}

pub struct ShapeInteract<'a> {
    shape: &'a mut Shape<Point>,
}

impl<'a> ShapeInteract<'a> {
    pub fn new(shape: &'a mut Shape<Point>) -> Self {
        Self { shape }
    }

    fn do_point_action(&mut self, index: usize, action: PointAction) {
        match action {
            PointAction::Click => {
                if index == 0 && self.shape.len() >= 2 {
                    self.shape.toggle_close();
                }
            }
            PointAction::Delete => {
                self.shape.remove(index);
            }
            PointAction::ConvertToCorner => {
                let old = &self.shape.points()[index];
                let mut p = CornerPoint::new(*old.point());
                if let Some(in_ctrl) = old.in_ctrl() {
                    p = p.with_in_ctrl(in_ctrl.into_owned())
                }
                if let Some(out_ctrl) = old.out_ctrl() {
                    p = p.with_out_ctrl(out_ctrl.into_owned())
                }
                self.shape.replace(index, p.into());
            }
            PointAction::ConvertToSmooth => {
                let old = &self.shape.points()[index];
                let point = *old.point();

                let mut theta: f64 = 0.0;
                let mut in_length = 10.0;
                let mut out_length = 10.0;
                let mut calculated = false;
                // if current point have any ctrl point, we calculate out ctrl direction
                // from current ctrl point, out ctrl takes priority(overrides in ctrl result).
                if let Some(in_ctrl) = old.in_ctrl() {
                    (in_length, theta) = point.minus(in_ctrl.as_ref()).polar();
                    calculated = true;
                }
                if let Some(out_ctrl) = old.out_ctrl() {
                    (out_length, theta) = out_ctrl.minus(&point).polar();
                    calculated = true;
                }
                // if current point do not have any ctrl points,
                // we lookup next point, and use this direction as out ctrl direction
                if !calculated && self.shape.len() > 1 {
                    let next = if index + 1 == self.shape.len() {
                        0
                    } else {
                        index + 1
                    };
                    (_, theta) = self.shape.points()[next].point().minus(&point).polar();
                }
                // replace
                self.shape.replace(
                    index,
                    SmoothPoint::new(point, theta, in_length, out_length).into(),
                );
            }
        }
    }

    pub fn snap_to_curve_with_radius(
        &self, pos: Pos2, transform: &PlotTransform, radius: f64,
    ) -> (Point, Option<Nearest<Point>>) {
        let target = transform.value_from_position(pos).into();
        let mut nearest = self.shape.nearest_point_on_curves(&target, false);

        if let Some(ref n) = nearest {
            let npos = transform.position_from_point(&n.point.0);
            if pos.distance(npos) > radius as f32 {
                nearest.take();
            }
        }

        (target, nearest)
    }

    fn do_clicked<R>(&mut self, response: &PlotResponse<R>) {
        if response.response.clicked() {
            if let Some(pos) = response.response.interact_pointer_pos() {
                let (target, nearest) =
                    self.snap_to_curve_with_radius(pos, &response.transform, 12.0);

                if let Some(n) = nearest {
                    self.shape.insert_on_curve(n.index, n.t);
                } else if !self.shape.closed() {
                    self.shape.push(CornerPoint::new(target).into());
                }
            }
        }
    }

    pub fn interact<R>(&mut self, ui: &mut Ui, id: Id, response: &PlotResponse<R>) {
        let mut act = None;

        for (i, point) in self.shape.points_mut().iter_mut().enumerate() {
            let mut interact = CurvePointInteract(point);
            if let Some(action) = interact.interact(ui, id.with(i), &response.transform) {
                act.replace((i, action));
            }
        }

        if let Some((index, action)) = act {
            self.do_point_action(index, action);
        }

        self.do_clicked(response);
    }
}
