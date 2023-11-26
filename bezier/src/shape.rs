use eframe::{
    egui::{CollapsingHeader, Id, Ui},
    epaint::Pos2,
};
use egui_plot::{PlotPoint, PlotResponse, PlotTransform, PlotUi};

use crate::{
    bezier::Bezier,
    line::LineSegment,
    option::{LinePlotOption, CURVE},
    point::{CornerPoint, CurvePoint, PlotPointExt, PointAction, SmoothPoint},
};

#[derive(Default)]
pub struct Shape {
    points: Vec<CurvePoint>,
    close: bool,
}

pub enum Segment<'a> {
    Line(LineSegment<'a>),
    Bezier(Bezier<'a>),
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

    pub fn nearest_to(&self, target: &PlotPoint) -> Option<(PlotPoint, f64)> {
        match self {
            Self::Bezier(b) => b.nearest_to(target),
            Self::Line(l) => l.nearest_to(target),
        }
    }

    pub fn plot(&self, plot: &mut PlotUi, opt: LinePlotOption) {
        match self {
            Self::Bezier(b) => b.plot(plot, opt),
            Self::Line(l) => l.plot(plot, opt),
        }
    }
}

impl Shape {
    pub fn toggle_close(&mut self) {
        self.close = !self.close;
    }

    pub fn segments(&self) -> impl Iterator<Item = Segment<'_>> {
        let mut close_returned = false;

        self.points
            .windows(2)
            .map(|curve| Segment::new(&curve[0], &curve[1]))
            .chain(std::iter::from_fn(move || {
                if !close_returned && self.close && self.points.len() >= 2 {
                    close_returned = true;
                    Some(Segment::new(
                        self.points.last().unwrap(),
                        self.points.first().unwrap(),
                    ))
                } else {
                    None
                }
            }))
    }

    pub fn plot(&self, plot: &mut PlotUi) {
        if self.points.is_empty() {
            return;
        }

        for point in &self.points {
            point.plot(plot);
        }

        for segment in self.segments() {
            segment.plot(plot, CURVE);
        }
    }

    pub fn interact<R>(&mut self, ui: &mut Ui, id: Id, response: PlotResponse<R>) {
        let mut act = None;

        for (i, point) in self.points.iter_mut().enumerate() {
            if let Some(action) = point.interact(ui, id.with(i), response.transform) {
                act.replace((i, action));
            }
        }

        if let Some((index, action)) = act {
            match action {
                PointAction::Click => {
                    if index == 0 && self.points.len() >= 2 {
                        self.toggle_close();
                    }
                }
                PointAction::Delete => {
                    self.points.remove(index);
                }
                PointAction::ConvertToCorner => {
                    let old = &self.points[index];
                    let mut p = CornerPoint::new(*old.point());
                    if let Some(in_ctrl) = old.in_ctrl() {
                        p = p.with_in_ctrl(in_ctrl.into_owned())
                    }
                    if let Some(out_ctrl) = old.out_ctrl() {
                        p = p.with_out_ctrl(out_ctrl.into_owned())
                    }
                    self.points[index] = p.into();
                }
                PointAction::ConvertToSmooth => {
                    let old = &self.points[index];
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
                    if !calculated && self.points.len() > 1 {
                        let next = if index == self.points.len() - 1 {
                            0
                        } else {
                            index + 1
                        };
                        (_, theta) = self.points[next].point().minus(&point).polar();
                    }
                    // replace
                    self.points[index] =
                        SmoothPoint::new(point, theta, in_length, out_length).into();
                }
            }
        }
        if response.response.clicked() {
            let Some(pos) = response.response.interact_pointer_pos() else {
                return;
            };

            let target = response.transform.value_from_position(pos);

            let mut inserted = self.snap_to_segment(&target, pos, 12.0, response.transform);

            if inserted.is_none() && !self.close {
                inserted.replace((self.points.len() - 1, target));
            }

            if let Some((index, point)) = inserted {
                self.points
                    .insert(index + 1, CornerPoint::new(point).into());
            }
        }
    }

    pub fn snap_to_segment(
        &self, target: &PlotPoint, pos: Pos2, radius: f64, transform: PlotTransform,
    ) -> Option<(usize, PlotPoint)> {
        let mut inserted = self.nearest_point_on_segment(target);

        if let Some((_, p, _)) = inserted {
            let p_pos = transform.position_from_point(&p);
            if pos.distance(p_pos) > radius as f32 {
                inserted.take();
            }
        }

        inserted.map(|(i, p, _)| (i, p))
    }

    pub fn nearest_point_on_segment(&self, target: &PlotPoint) -> Option<(usize, PlotPoint, f64)> {
        // TODO: bounding box clip
        self.segments()
            .enumerate()
            .flat_map(|(i, s)| s.nearest_to(target).map(|(p, d)| (i, p, d)))
            .min_by(|(_, _, d), (_, _, d2)| d.total_cmp(d2))
    }

    pub fn controls(&mut self, ui: &mut Ui, id: Id) {
        let mut deleted = None;
        for (i, p) in self.points.iter_mut().enumerate() {
            if let Some(Some(del)) = CollapsingHeader::new(i.to_string().as_str())
                .id_source(id.with(i))
                .show(ui, |ui| {
                    p.ui(ui);

                    if ui.button("Delete").clicked() {
                        return Some(i);
                    }

                    None
                })
                .body_returned
            {
                deleted.replace(del);
            }
        }

        if let Some(del) = deleted {
            self.points.remove(del);
        }
    }
}

impl FromIterator<CurvePoint> for Shape {
    fn from_iter<T: IntoIterator<Item = CurvePoint>>(iter: T) -> Self {
        Self {
            points: iter.into_iter().collect(),
            close: true,
        }
    }
}
