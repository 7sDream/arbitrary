use bezier::{CornerPoint, CurvePoint, Nearest, Point, PointExt, Shape, SmoothPoint};
use eframe::{
    egui::{CollapsingHeader, DragValue, Id, PointerButton, Response, Sense, Slider, Ui},
    epaint::{Pos2, Rect, Vec2},
};
use egui_plot::{PlotPoint, PlotResponse, PlotTransform};

use crate::configure::{self, CurvePointPlotConfig};

trait BezierPointExt {
    fn to_plot_point(&self) -> PlotPoint;

    fn from_plot_point(p: PlotPoint) -> Self;
}

impl BezierPointExt for Point {
    fn to_plot_point(&self) -> PlotPoint {
        PlotPoint {
            x: self.0,
            y: self.1,
        }
    }

    fn from_plot_point(p: PlotPoint) -> Self {
        [p.x, p.y].into()
    }
}

struct PointInteract {
    transform: PlotTransform,
    position: Pos2,
    response: Option<Response>,
}

impl PointInteract {
    pub fn new(point: &Point, id: Id, ui: &Ui, transform: PlotTransform, size: f64) -> Self {
        let pp = [point.0, point.1].into();

        let position = transform.position_from_point(&pp);

        let mut result = Self {
            transform,
            position,
            response: None,
        };

        let bound = transform.bounds();
        if !bound.is_valid() {
            return result;
        }

        let [x_min, y_min] = bound.min();
        let [x_max, y_max] = bound.max();
        let half = size / 2.0;

        if point.x() + half < x_min
            || point.x() - half > x_max
            || point.y() + half < y_min
            || point.y() - half > y_max
        {
            return result;
        }

        let rect = Rect::from_center_size(result.position, Vec2::splat(size as f32));
        let response = ui.interact(rect, id, Sense::click_and_drag());

        result.response.replace(response);

        result
    }

    pub fn drag(&mut self, p: &mut Point) -> bool {
        if let Some(delta) = self.drag_delta() {
            p.0 += delta.0;
            p.1 += delta.1;
            return true;
        }

        false
    }

    pub fn drag_delta(&mut self) -> Option<Point> {
        if let Some(resp) = self.response.as_mut() {
            if resp.dragged_by(PointerButton::Primary) {
                let delta_pos = resp.drag_delta();
                let d = self.transform.dvalue_dpos();
                return Some([delta_pos.x as f64 * d[0], delta_pos.y as f64 * d[1]].into());
            }
        }

        None
    }

    pub fn clicked(&self) -> bool {
        self.response
            .as_ref()
            .map(|r| r.clicked())
            .unwrap_or_default()
    }

    pub fn context_menu(&mut self, add_contents: impl FnOnce(&mut Ui)) {
        if let Some(resp) = self.response.take() {
            self.response.replace(resp.context_menu(add_contents));
        }
    }
}

pub fn controls_point(p: &mut Point, ui: &mut Ui, text: &str) -> bool {
    ui.horizontal(|ui| {
        let mut changed = false;

        ui.label(text);

        if ui
            .add(
                DragValue::new(&mut p.0)
                    .prefix("x: ")
                    .update_while_editing(false),
            )
            .changed()
        {
            changed = true;
        }

        if ui
            .add(
                DragValue::new(&mut p.1)
                    .prefix("y: ")
                    .update_while_editing(false),
            )
            .changed()
        {
            changed = true;
        };

        changed
    })
    .inner
}

pub enum PointAction {
    Click,
    Delete,
    ConvertToCorner,
    ConvertToSmooth,
}

struct CornerPointInteract<'a>(&'a mut CornerPoint);

impl<'a> CornerPointInteract<'a> {
    fn point_interact(
        &mut self, ui: &mut Ui, id: Id, transform: &PlotTransform, opt: &CurvePointPlotConfig,
    ) -> Option<PointAction> {
        let mut action = None;

        let mut act = PointInteract::new(
            self.0.point(),
            id.with("point"),
            ui,
            *transform,
            opt.point.size,
        );

        if let Some(delta) = act.drag_delta() {
            self.0.move_delta(delta, true);
        }

        act.context_menu(|ui| {
            self.controls(ui);

            if !self.0.has_in_ctrl() || !self.0.has_out_ctrl() {
                ui.menu_button("Add", |ui| {
                    ui.add_enabled_ui(!self.0.has_in_ctrl(), |ui| {
                        if ui.button("In ctrl point").clicked() {
                            let p = [self.0.point().x() - 10.0, self.0.point().y()].into();
                            self.0.update_in_ctrl(p);
                            ui.close_menu();
                        }
                    });
                    ui.add_enabled_ui(!self.0.has_out_ctrl(), |ui| {
                        if ui.button("Out ctrl point").clicked() {
                            let p = [self.0.point().x() + 10.0, self.0.point().y()].into();
                            self.0.update_out_ctrl(p);
                            ui.close_menu();
                        }
                    });
                });
            }

            ui.menu_button("Delete", |ui| {
                ui.add_enabled_ui(self.0.in_ctrl().is_some(), |ui| {
                    if ui.button("In ctrl point").clicked() {
                        self.0.remove_in_ctrl();
                        ui.close_menu();
                    }
                });

                ui.add_enabled_ui(self.0.out_ctrl().is_some(), |ui| {
                    if ui.button("Out ctrl point").clicked() {
                        self.0.remove_out_ctrl();
                        ui.close_menu();
                    }
                });

                if ui.button("Point").clicked() {
                    action.replace(PointAction::Delete);
                    ui.close_menu();
                }
            });

            if ui.button("Convert to smooth point").clicked() {
                action.replace(PointAction::ConvertToSmooth);
                ui.close_menu();
            }
        });

        if act.clicked() {
            action.replace(PointAction::Click);
        }

        action
    }

    fn ctrl_interact(
        &mut self, ui: &mut Ui, id: Id, transform: &PlotTransform, opt: &CurvePointPlotConfig,
    ) {
        let mut delete_in = false;
        if let Some(p) = self.0.in_ctrl_mut() {
            let mut in_act = PointInteract::new(p, id.with("in"), ui, *transform, opt.in_ctrl.size);
            in_act.drag(p);
            in_act.context_menu(|ui| {
                controls_point(p, ui, "In ctrl");

                if ui.button("Delete").clicked() {
                    delete_in = true;
                    ui.close_menu();
                }
            });
        }
        if delete_in {
            self.0.remove_in_ctrl();
        }

        let mut delete_out = false;
        if let Some(p) = self.0.out_ctrl_mut() {
            let mut out_act =
                PointInteract::new(p, id.with("out"), ui, *transform, opt.out_ctrl.size);
            out_act.drag(p);
            out_act.context_menu(|ui| {
                controls_point(p, ui, "Out ctrl");

                if ui.button("Delete").clicked() {
                    delete_out = true;
                    ui.close_menu();
                }
            });
        }
        if delete_out {
            self.0.remove_out_ctrl();
        }
    }

    pub fn interact(
        &mut self, ui: &mut Ui, id: Id, transform: &PlotTransform,
    ) -> Option<PointAction> {
        let conf = &configure::read();
        let opt = &conf.plot.cornel;

        let action = self.point_interact(ui, id, transform, opt);

        if conf.view.show_ctrl {
            self.ctrl_interact(ui, id, transform, opt);
        }

        action
    }

    pub fn controls(&mut self, ui: &mut Ui) {
        let mut current = *self.0.point();
        if controls_point(&mut current, ui, "Point") {
            self.0.move_to(current, true);
        }

        if let Some(p) = self.0.in_ctrl_mut() {
            controls_point(p, ui, "In ctrl");
        }
        if let Some(p) = self.0.out_ctrl_mut() {
            controls_point(p, ui, "Out ctrl");
        }
    }
}

struct SmoothPointInteract<'a>(&'a mut SmoothPoint);

impl<'a> SmoothPointInteract<'a> {
    fn ctrl_interact(
        &mut self, ui: &mut Ui, id: Id, transform: &PlotTransform, opt: &CurvePointPlotConfig,
    ) {
        let mut in_ctrl = self.0.in_ctrl();
        let mut in_act =
            PointInteract::new(&in_ctrl, id.with("in"), ui, *transform, opt.in_ctrl.size);
        if in_act.drag(&mut in_ctrl) {
            self.0.move_in_ctrl_to(&in_ctrl);
        }
        in_act.context_menu(|ui| {
            self.theta_control(ui);
            self.in_length_control(ui);

            if ui.button("Same length as out").clicked() {
                self.0.update_in_length(self.0.out_length());
                ui.close_menu();
            }
        });

        let mut out_ctrl = self.0.out_ctrl();
        let mut out_act =
            PointInteract::new(&out_ctrl, id.with("out"), ui, *transform, opt.out_ctrl.size);
        if out_act.drag(&mut out_ctrl) {
            self.0.move_out_ctrl_to(&out_ctrl);
        }
        out_act.context_menu(|ui| {
            self.theta_control(ui);
            self.out_length_control(ui);

            if ui.button("Same length as in").clicked() {
                self.0.update_out_length(self.0.in_length());
                ui.close_menu();
            }
        });
    }

    fn point_interact(
        &mut self, ui: &mut Ui, id: Id, transform: &PlotTransform, opt: &CurvePointPlotConfig,
    ) -> Option<PointAction> {
        let mut action = None;

        let mut act = PointInteract::new(
            self.0.point(),
            id.with("point"),
            ui,
            *transform,
            opt.point.size,
        );

        act.drag(self.0.point_mut());

        act.context_menu(|ui| {
            self.controls(ui);

            if ui.button("Convert to corner point").clicked() {
                action.replace(PointAction::ConvertToCorner);
                ui.close_menu();
            }

            if ui.button("Delete").clicked() {
                action.replace(PointAction::Delete);
                ui.close_menu();
            }
        });
        if act.clicked() {
            action.replace(PointAction::Click);
        }

        action
    }

    pub fn interact(
        &mut self, ui: &mut Ui, id: Id, transform: &PlotTransform,
    ) -> Option<PointAction> {
        let conf = &configure::read();
        let opt = &conf.plot.smooth;

        let action = self.point_interact(ui, id, transform, opt);

        if conf.view.show_ctrl {
            self.ctrl_interact(ui, id, transform, opt);
        }

        action
    }

    pub fn point_control(&mut self, ui: &mut Ui) {
        controls_point(self.0.point_mut(), ui, "Point");
    }

    pub fn theta_control(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Theta: ");

            let mut theta = self.0.theta();
            if ui
                .add(
                    Slider::new(&mut theta, 0.0..=359.999)
                        .smart_aim(true)
                        .suffix("°"),
                )
                .changed()
            {
                self.0.update_theta(theta);
            }
        });
    }

    pub fn in_length_control(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("In ctrl: ");
            let mut l = self.0.in_length();
            if ui
                .add(
                    Slider::new(&mut l, 0.0..=100.0)
                        .smart_aim(true)
                        .clamp_to_range(false),
                )
                .changed()
            {
                self.0.update_in_length(l);
            }
        });
    }

    pub fn out_length_control(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Out ctrl: ");
            let mut l = self.0.out_length();
            if ui
                .add(
                    Slider::new(&mut l, 0.0..=100.0)
                        .smart_aim(true)
                        .clamp_to_range(false),
                )
                .changed()
            {
                self.0.update_out_length(l);
            }
        });
    }

    pub fn controls(&mut self, ui: &mut Ui) {
        self.point_control(ui);
        self.theta_control(ui);
        self.in_length_control(ui);
        self.out_length_control(ui);
    }
}

struct CurvePointInteract<'a>(&'a mut CurvePoint);

impl<'a> CurvePointInteract<'a> {
    pub fn interact(
        &mut self, ui: &mut Ui, id: Id, transform: &PlotTransform,
    ) -> Option<PointAction> {
        match self.0 {
            CurvePoint::Corner(c) => CornerPointInteract(c).interact(ui, id, transform),
            CurvePoint::Smooth(s) => SmoothPointInteract(s).interact(ui, id, transform),
        }
    }

    pub fn controls(&mut self, ui: &mut Ui) {
        match self.0 {
            CurvePoint::Corner(c) => CornerPointInteract(c).controls(ui),
            CurvePoint::Smooth(s) => SmoothPointInteract(s).controls(ui),
        }
    }
}

pub struct ShapeInteract<'a> {
    shape: &'a mut Shape,
}

impl<'a> ShapeInteract<'a> {
    pub fn new(shape: &'a mut Shape) -> Self {
        Self { shape }
    }

    pub fn controls(&mut self, ui: &mut Ui, id: Id) {
        let mut deleted = None;
        for (i, p) in self.shape.points_mut().iter_mut().enumerate() {
            if let Some(Some(del)) = CollapsingHeader::new(i.to_string().as_str())
                .id_source(id.with(i))
                .show(ui, |ui| {
                    CurvePointInteract(p).controls(ui);

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
            self.shape.remove(del);
        }
    }

    fn do_action(&mut self, index: usize, action: PointAction) {
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
        &self, target: &PlotPoint, pos: Pos2, transform: &PlotTransform, radius: f64,
    ) -> Option<Nearest> {
        let mut nearest = self
            .shape
            .nearest_point_on_curves(&Point::from_plot_point(*target), false);

        if let Some(ref n) = nearest {
            let p_pos = transform.position_from_point(&n.point.to_plot_point());
            if pos.distance(p_pos) > radius as f32 {
                nearest.take();
            }
        }

        nearest
    }

    fn insert_nearest_without_calculation(&mut self, target: Point, nearest: Option<Nearest>) {
        if let Some(n) = nearest {
            self.shape.insert_on_curve(n.index, n.t);
        } else if !self.shape.closed() {
            self.shape.push(CornerPoint::new(target).into());
        }
    }

    pub fn interact(
        &mut self, ui: &mut Ui, id: Id,
        response: PlotResponse<Option<(PlotPoint, Option<Nearest>)>>,
    ) {
        let mut act = None;

        for (i, point) in self.shape.points_mut().iter_mut().enumerate() {
            let mut interact = CurvePointInteract(point);
            if let Some(action) = interact.interact(ui, id.with(i), &response.transform) {
                act.replace((i, action));
            }
        }

        if let Some((index, action)) = act {
            self.do_action(index, action);
        }

        if let Some((target, nearest)) = response.inner {
            self.insert_nearest_without_calculation(Point::from_plot_point(target), nearest)
        }
    }
}
