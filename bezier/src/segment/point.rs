use std::borrow::Cow;

use eframe::{
    egui::{DragValue, Id, PointerButton, Response, Sense, Slider, Ui},
    epaint::{Pos2, Rect, Vec2},
};
use egui_plot::{PlotPoint, PlotPoints, PlotTransform, PlotUi, Points};

use super::LineSegment;
use crate::option::{PointPlotOption, CORNEL_POINT, SMOOTH_POINT};

pub trait PlotPointExt {
    fn x(&self) -> f64;
    fn y(&self) -> f64;

    fn plot(&self, plot: &mut PlotUi, opt: PointPlotOption) {
        plot.points(
            Points::new(PlotPoints::Owned(vec![[self.x(), self.y()].into()]))
                .shape(opt.mark)
                .filled(true)
                .radius(opt.size as f32 / 2.0)
                .color(opt.color),
        )
    }

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

    fn negative(&self) -> PlotPoint {
        PlotPoint {
            x: -self.x(),
            y: -self.y(),
        }
    }

    fn minus(&self, rhs: &PlotPoint) -> PlotPoint {
        PlotPoint {
            x: self.x() - rhs.x,
            y: self.y() - rhs.y,
        }
    }

    fn plus(&self, rhs: &PlotPoint) -> PlotPoint {
        PlotPoint {
            x: self.x() + rhs.x,
            y: self.y() + rhs.x,
        }
    }

    fn move_follow(&self, dir: f64, length: f64) -> PlotPoint {
        PlotPoint {
            x: self.x() + dir.to_radians().cos() * length,
            y: self.y() + dir.to_radians().sin() * length,
        }
    }
}

struct PointInteract {
    transform: PlotTransform,
    position: Pos2,
    response: Option<Response>,
}

impl PointInteract {
    pub fn new(point: &PlotPoint, id: Id, ui: &Ui, transform: PlotTransform, size: f64) -> Self {
        let position = transform.position_from_point(point);

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

        if point.x + half < x_min
            || point.x - half > x_max
            || point.y + half < y_min
            || point.y - half > y_max
        {
            return result;
        }

        let rect = Rect::from_center_size(result.position, Vec2::splat(size as f32));
        let response = ui.interact(rect, id, Sense::click_and_drag());

        result.response.replace(response);

        result
    }

    pub fn drag(&mut self, p: &mut PlotPoint) -> bool {
        if let Some(resp) = self.response.as_mut() {
            if resp.dragged_by(PointerButton::Primary) {
                let sp = self.transform.position_from_point(p) + resp.drag_delta();
                *p = self.transform.value_from_position(sp);
                return true;
            }
        }

        false
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

pub fn controls(p: &mut PlotPoint, ui: &mut Ui, text: &str) {
    ui.horizontal(|ui| {
        ui.label(text);
        ui.add(
            DragValue::new(&mut p.x)
                .prefix("x: ")
                .update_while_editing(false),
        );
        ui.add(
            DragValue::new(&mut p.y)
                .prefix("y: ")
                .update_while_editing(false),
        );
    });
}

impl PlotPointExt for PlotPoint {
    fn x(&self) -> f64 {
        self.x
    }

    fn y(&self) -> f64 {
        self.y
    }
}

pub enum PointAction {
    Click,
    Delete,
    ConvertToCorner,
    ConvertToSmooth,
}

#[derive(Clone)]
pub struct CornerPoint {
    in_ctrl: Option<PlotPoint>,
    point: PlotPoint,
    out_ctrl: Option<PlotPoint>,
}

impl CornerPoint {
    pub fn new(point: PlotPoint) -> Self {
        Self {
            point,
            in_ctrl: None,
            out_ctrl: None,
        }
    }

    pub fn with_in_ctrl(mut self, point: PlotPoint) -> Self {
        self.in_ctrl.replace(point);
        self
    }

    pub fn with_out_ctrl(mut self, point: PlotPoint) -> Self {
        self.out_ctrl.replace(point);
        self
    }

    pub fn plot(&self, plot: &mut PlotUi) {
        self.point.plot(plot, CORNEL_POINT.point);
        if let Some(p) = &self.in_ctrl {
            p.plot(plot, CORNEL_POINT.in_ctrl);
            LineSegment::new(&self.point, p).plot(plot, CORNEL_POINT.in_ctrl_link);
        }
        if let Some(p) = &self.out_ctrl {
            p.plot(plot, CORNEL_POINT.out_ctrl);
            LineSegment::new(&self.point, p).plot(plot, CORNEL_POINT.out_ctrl_link);
        }
    }

    pub fn interact(
        &mut self, ui: &mut Ui, id: Id, transform: PlotTransform,
    ) -> Option<PointAction> {
        let mut action = None;

        let mut p_act = PointInteract::new(
            &self.point,
            id.with("point"),
            ui,
            transform,
            CORNEL_POINT.point.size,
        );
        p_act.drag(&mut self.point);
        p_act.context_menu(|ui| {
            self.controls(ui);

            if self.in_ctrl.is_none() || self.out_ctrl.is_none() {
                ui.menu_button("Add", |ui| {
                    ui.add_enabled_ui(self.in_ctrl.is_none(), |ui| {
                        if ui.button("In ctrl point").clicked() {
                            self.in_ctrl.replace(PlotPoint {
                                x: self.point.x - 10.0,
                                y: self.point.y,
                            });
                            ui.close_menu();
                        }
                    });
                    ui.add_enabled_ui(self.out_ctrl.is_none(), |ui| {
                        if ui.button("Out ctrl point").clicked() {
                            self.out_ctrl.replace(PlotPoint {
                                x: self.point.x + 10.0,
                                y: self.point.y,
                            });
                            ui.close_menu();
                        }
                    });
                });
            }

            ui.menu_button("Delete", |ui| {
                ui.add_enabled_ui(self.in_ctrl.is_some(), |ui| {
                    if ui.button("In ctrl point").clicked() {
                        self.in_ctrl.take();
                        ui.close_menu();
                    }
                });

                ui.add_enabled_ui(self.out_ctrl.is_some(), |ui| {
                    if ui.button("Out ctrl point").clicked() {
                        self.out_ctrl.take();
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
        if p_act.clicked() {
            action.replace(PointAction::Click);
        }

        let mut delete_in = false;
        if let Some(p) = self.in_ctrl.as_mut() {
            p_act.drag(p); // ctrl point move follows main point
            let mut in_act =
                PointInteract::new(p, id.with("in"), ui, transform, CORNEL_POINT.in_ctrl.size);
            in_act.drag(p);
            in_act.context_menu(|ui| {
                controls(p, ui, "In ctrl");

                if ui.button("Delete").clicked() {
                    delete_in = true;
                    ui.close_menu();
                }
            });
        }
        if delete_in {
            self.in_ctrl.take();
        }

        let mut delete_out = false;
        if let Some(p) = self.out_ctrl.as_mut() {
            p_act.drag(p); // ctrl point move follows main point
            let mut out_act =
                PointInteract::new(p, id.with("out"), ui, transform, CORNEL_POINT.out_ctrl.size);
            out_act.drag(p);
            out_act.context_menu(|ui| {
                controls(p, ui, "Out ctrl");

                if ui.button("Delete").clicked() {
                    delete_out = true;
                    ui.close_menu();
                }
            });
        }
        if delete_out {
            self.out_ctrl.take();
        }

        action
    }

    pub fn controls(&mut self, ui: &mut Ui) {
        controls(&mut self.point, ui, "Point");
        if let Some(p) = self.in_ctrl.as_mut() {
            controls(p, ui, "In ctrl");
        }
        if let Some(p) = self.out_ctrl.as_mut() {
            controls(p, ui, "Out ctrl");
        }
    }
}

#[derive(Clone)]
pub struct SmoothPoint {
    point: PlotPoint,
    theta: f64,
    in_length: f64,
    out_length: f64,
}

impl SmoothPoint {
    pub fn new(point: PlotPoint, theta: f64, in_length: f64, out_length: f64) -> Self {
        Self::new_unchecked(point, theta % (360.0), in_length.abs(), out_length.abs())
    }

    fn new_unchecked(point: PlotPoint, rad: f64, in_length: f64, out_length: f64) -> Self {
        Self {
            point,
            theta: rad,
            in_length,
            out_length,
        }
    }

    pub fn horizontal(point: PlotPoint, in_length: f64, out_length: f64) -> Self {
        Self::new_unchecked(point, 0.0, in_length.abs(), out_length.abs())
    }

    #[allow(dead_code)] // TODO: remove this
    pub fn vertical(point: PlotPoint, in_length: f64, out_length: f64) -> Self {
        Self::new_unchecked(point, 90.0, in_length.abs(), out_length.abs())
    }

    pub fn in_ctrl(&self) -> PlotPoint {
        self.point.move_follow(self.theta + 180.0, self.in_length)
    }

    pub fn out_ctrl(&self) -> PlotPoint {
        self.point.move_follow(self.theta, self.out_length)
    }

    pub fn update_in_ctrl(&mut self, val: &PlotPoint) {
        let v = self.point.minus(&val);
        (self.in_length, self.theta) = v.polar();
    }

    pub fn update_out_ctrl(&mut self, val: &PlotPoint) {
        let v = val.minus(&self.point);
        (self.out_length, self.theta) = v.polar();
    }

    pub fn plot(&self, plot: &mut PlotUi) {
        self.point.plot(plot, SMOOTH_POINT.point);

        let in_ctrl = self.in_ctrl();
        let out_ctrl = self.out_ctrl();

        in_ctrl.plot(plot, SMOOTH_POINT.in_ctrl);
        out_ctrl.plot(plot, SMOOTH_POINT.out_ctrl);

        LineSegment::new(&self.point, &in_ctrl).plot(plot, SMOOTH_POINT.in_ctrl_link);
        LineSegment::new(&self.point, &out_ctrl).plot(plot, SMOOTH_POINT.out_ctrl_link);
    }

    pub fn interact(
        &mut self, ui: &mut Ui, id: Id, transform: PlotTransform,
    ) -> Option<PointAction> {
        let mut action = None;

        let mut p_act = PointInteract::new(
            &self.point,
            id.with("point"),
            ui,
            transform,
            SMOOTH_POINT.point.size,
        );
        p_act.drag(&mut self.point);
        p_act.context_menu(|ui| {
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
        if p_act.clicked() {
            action.replace(PointAction::Click);
        }

        let mut in_ctrl = self.in_ctrl();
        let mut in_act = PointInteract::new(
            &in_ctrl,
            id.with("in"),
            ui,
            transform,
            SMOOTH_POINT.in_ctrl.size,
        );
        if in_act.drag(&mut in_ctrl) {
            self.update_in_ctrl(&in_ctrl);
        }
        in_act.context_menu(|ui| {
            self.theta_control(ui);
            self.in_length_control(ui);

            if ui.button("Same length as out").clicked() {
                self.in_length = self.out_length;
                ui.close_menu();
            }
        });

        let mut out_ctrl = self.out_ctrl();
        let mut out_act = PointInteract::new(
            &out_ctrl,
            id.with("out"),
            ui,
            transform,
            SMOOTH_POINT.out_ctrl.size,
        );
        if out_act.drag(&mut out_ctrl) {
            self.update_out_ctrl(&out_ctrl);
        }
        out_act.context_menu(|ui| {
            self.theta_control(ui);
            self.out_length_control(ui);

            if ui.button("Same length as in").clicked() {
                self.out_length = self.in_length;
                ui.close_menu();
            }
        });

        action
    }

    pub fn point_control(&mut self, ui: &mut Ui) {
        controls(&mut self.point, ui, "Point");
    }

    pub fn theta_control(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Theta: ");
            ui.add(
                Slider::new(&mut self.theta, 0.0..=360.0)
                    .smart_aim(true)
                    .suffix("°"),
            );
        });
    }

    pub fn in_length_control(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("In ctrl: ");
            if ui
                .add(
                    Slider::new(&mut self.in_length, 0.0..=100.0)
                        .smart_aim(true)
                        .clamp_to_range(false),
                )
                .changed()
            {
                self.in_length = self.in_length.abs();
            }
        });
    }

    pub fn out_length_control(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Out ctrl: ");
            if ui
                .add(
                    Slider::new(&mut self.out_length, 0.0..=100.0)
                        .smart_aim(true)
                        .clamp_to_range(false),
                )
                .changed()
            {
                self.out_length = self.out_length.abs();
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

#[derive(Clone)]
pub enum CurvePoint {
    Corner(CornerPoint),
    Smooth(SmoothPoint),
}

impl CurvePoint {
    pub fn point(&self) -> &PlotPoint {
        match self {
            Self::Corner(c) => &c.point,
            Self::Smooth(s) => &s.point,
        }
    }

    pub fn in_ctrl<'a>(&'a self) -> Option<Cow<'a, PlotPoint>> {
        match self {
            Self::Corner(c) => c.in_ctrl.as_ref().map(Cow::Borrowed),
            Self::Smooth(s) => Some(Cow::Owned(s.in_ctrl())),
        }
    }

    pub fn out_ctrl<'a>(&'a self) -> Option<Cow<'a, PlotPoint>> {
        match self {
            Self::Corner(c) => c.out_ctrl.as_ref().map(Cow::Borrowed),
            Self::Smooth(s) => Some(Cow::Owned(s.out_ctrl())),
        }
    }

    pub fn set_in_ctrl(&mut self, val: PlotPoint) {
        match self {
            Self::Corner(c) => {
                c.in_ctrl.replace(val);
            }
            Self::Smooth(s) => s.update_in_ctrl(&val),
        };
    }

    pub fn set_out_ctrl(&mut self, val: PlotPoint) {
        match self {
            Self::Corner(c) => {
                c.out_ctrl.replace(val);
            }
            Self::Smooth(s) => s.update_out_ctrl(&val),
        };
    }

    pub fn plot(&self, plot: &mut PlotUi) {
        match self {
            Self::Corner(c) => c.plot(plot),
            Self::Smooth(s) => s.plot(plot),
        }
    }

    pub fn interact(
        &mut self, ui: &mut Ui, id: Id, transform: PlotTransform,
    ) -> Option<PointAction> {
        match self {
            Self::Corner(c) => c.interact(ui, id, transform),
            Self::Smooth(s) => s.interact(ui, id, transform),
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        match self {
            Self::Corner(c) => c.controls(ui),
            Self::Smooth(s) => s.controls(ui),
        }
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
