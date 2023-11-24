use std::{borrow::Cow, f64::consts::PI};

use eframe::{
    egui::{DragValue, Id, PointerButton, Sense, Slider, Ui},
    epaint::{Color32, Rect, Vec2},
};
use egui_plot::{MarkerShape, PlotPoint, PlotPoints, PlotTransform, PlotUi, Points};

use crate::{
    constants::{
        CORNEL_MARK, CTRL_1_COLOR, CTRL_2_COLOR, CTRL_LINK_LINE_COLOR, CTRL_MARK, MAIN_POINT_COLOR,
        SMOOTH_MARK,
    },
    line::LineSegment,
};

pub const MAIN_POINT_SIZE: f32 = 8.0;
pub const CTRL_POINT_SIZE: f32 = 6.0;
pub const POINT_DRAG_RADIUS: f32 = 8.0;

pub trait PlotPointExt {
    fn x(&self) -> f64;
    fn y(&self) -> f64;

    fn plot(&self, plot: &mut PlotUi, mark: MarkerShape, size: f32, color: Color32) {
        plot.points(
            Points::new(PlotPoints::Owned(vec![[self.x(), self.y()].into()]))
                .shape(mark)
                .filled(true)
                .radius(size)
                .color(color),
        )
    }

    fn length_from_origin(&self) -> f64 {
        let (x, y) = (self.x(), self.y());
        (x * x + y * y).sqrt()
    }

    // theta between [0, 2Pi]
    fn polar(&self) -> (f64, f64) {
        let (x, y) = (self.x(), self.y());

        if x == 0.0 && y == 0.0 {
            return (0.0, 0.0);
        }

        let r = self.length_from_origin();

        let mut theta = (x / self.length_from_origin()).acos();
        if y.is_sign_negative() {
            theta = 2.0 * PI - theta;
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
            x: self.x() + dir.cos() * length,
            y: self.y() + dir.sin() * length,
        }
    }
}

fn drag(p: &mut PlotPoint, id: Id, ui: &mut Ui, transform: PlotTransform) -> bool {
    let drag_rect_size = Vec2::splat(POINT_DRAG_RADIUS);
    let center = transform.position_from_point(p);
    let drag_rect = Rect::from_center_size(center, drag_rect_size);
    let resp = ui.interact(drag_rect, id, Sense::drag());

    if resp.dragged_by(PointerButton::Primary) {
        let delta = resp.drag_delta();
        let sp = center + delta;
        *p = transform.value_from_position(sp);
        true
    } else {
        false
    }
}

pub fn controls(p: &mut PlotPoint, ui: &mut Ui, text: &str) {
    ui.collapsing(text, |ui| {
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
        self.point
            .plot(plot, CORNEL_MARK, MAIN_POINT_SIZE, MAIN_POINT_COLOR);
        if let Some(p) = &self.in_ctrl {
            p.plot(plot, CTRL_MARK, CTRL_POINT_SIZE, CTRL_2_COLOR);
            LineSegment::new(&self.point, p).plot(plot, CTRL_LINK_LINE_COLOR, 1.0);
        }
        if let Some(p) = &self.out_ctrl {
            p.plot(plot, CTRL_MARK, CTRL_POINT_SIZE, CTRL_1_COLOR);
            LineSegment::new(&self.point, p).plot(plot, CTRL_LINK_LINE_COLOR, 1.0);
        }
    }

    pub fn drag(&mut self, ui: &mut Ui, id: Id, transform: PlotTransform) -> bool {
        if drag(&mut self.point, id.with("point"), ui, transform) {
            return true;
        }

        if let Some(p) = self.in_ctrl.as_mut() {
            if drag(p, id.with("ctrl1"), ui, transform) {
                return true;
            }
        }

        if let Some(p) = self.out_ctrl.as_mut() {
            if drag(p, id.with("ctrl2"), ui, transform) {
                return true;
            }
        }

        false
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        controls(&mut self.point, ui, "Point");
        if let Some(p) = self.in_ctrl.as_mut() {
            controls(p, ui, "In Ctrl");
        }
        if let Some(p) = self.out_ctrl.as_mut() {
            controls(p, ui, "Out Ctrl");
        }
    }
}

pub struct SmoothPoint {
    point: PlotPoint,
    theta: f64,
    in_length: f64,
    out_length: f64,
}

impl SmoothPoint {
    pub fn new(point: PlotPoint, theta: f64, length: f64) -> Self {
        Self::new_unchecked(point, theta % (2.0 * PI), length)
    }

    fn new_unchecked(point: PlotPoint, rad: f64, length: f64) -> Self {
        Self {
            point,
            theta: rad,
            in_length: length,
            out_length: length,
        }
    }

    pub fn horizontal(point: PlotPoint, length: f64) -> Self {
        Self::new_unchecked(point, 0.0, length)
    }

    pub fn vertical(point: PlotPoint, length: f64) -> Self {
        Self::new_unchecked(point, PI / 2.0, length)
    }

    pub fn in_ctrl(&self) -> PlotPoint {
        self.point.move_follow(self.theta + PI, self.in_length)
    }

    pub fn out_ctrl(&self) -> PlotPoint {
        self.point.move_follow(self.theta, self.out_length)
    }

    pub fn plot(&self, plot: &mut PlotUi) {
        self.point
            .plot(plot, SMOOTH_MARK, MAIN_POINT_SIZE, MAIN_POINT_COLOR);

        let in_ctrl = self.in_ctrl();
        let out_ctrl = self.out_ctrl();

        in_ctrl.plot(plot, CTRL_MARK, CTRL_POINT_SIZE, CTRL_2_COLOR);
        out_ctrl.plot(plot, CTRL_MARK, CTRL_POINT_SIZE, CTRL_1_COLOR);

        LineSegment::new(&self.point, &in_ctrl).plot(plot, CTRL_LINK_LINE_COLOR, 1.0);
        LineSegment::new(&self.point, &out_ctrl).plot(plot, CTRL_LINK_LINE_COLOR, 1.0);
    }

    pub fn drag(&mut self, ui: &mut Ui, id: Id, transform: PlotTransform) -> bool {
        if drag(&mut self.point, id.with("point"), ui, transform) {
            return true;
        }

        // TODO: shift key make horizontal or vertical
        let mut in_ctrl = self.in_ctrl();
        if drag(&mut in_ctrl, id.with("in_ctrl"), ui, transform) {
            let v = self.point.minus(&in_ctrl);
            (self.in_length, self.theta) = v.polar();
            return true;
        }

        let mut out_ctrl = self.out_ctrl();
        if drag(&mut out_ctrl, id.with("out_ctrl"), ui, transform) {
            let v = out_ctrl.minus(&self.point);
            (self.out_length, self.theta) = v.polar();
            return true;
        }

        false
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        controls(&mut self.point, ui, "Point");
        ui.add(
            Slider::new(&mut self.theta, 0.0..=2.0 * PI)
                .smart_aim(true)
                .prefix("Theta: ")
                .suffix(" rad"),
        );

        if ui
            .add(
                Slider::new(&mut self.in_length, 0.0..=100.0)
                    .smart_aim(true)
                    .clamp_to_range(false)
                    .prefix("In Ctrl: "),
            )
            .changed()
        {
            self.in_length = self.in_length.abs();
        }

        if ui
            .add(
                Slider::new(&mut self.out_length, 0.0..=100.0)
                    .smart_aim(true)
                    .clamp_to_range(false)
                    .prefix("Out Ctrl: "),
            )
            .changed()
        {
            self.out_length = self.out_length.abs();
        }
    }
}

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

    pub fn in_ctrl(&self) -> Option<Cow<'_, PlotPoint>> {
        match self {
            Self::Corner(c) => c.in_ctrl.as_ref().map(Cow::Borrowed),
            Self::Smooth(s) => Some(Cow::Owned(s.in_ctrl())),
        }
    }

    pub fn out_ctrl(&self) -> Option<Cow<'_, PlotPoint>> {
        match self {
            Self::Corner(c) => c.out_ctrl.as_ref().map(Cow::Borrowed),
            Self::Smooth(s) => Some(Cow::Owned(s.out_ctrl())),
        }
    }

    pub fn plot(&self, plot: &mut PlotUi) {
        match self {
            Self::Corner(c) => c.plot(plot),
            Self::Smooth(s) => s.plot(plot),
        }
    }

    pub fn drag(&mut self, ui: &mut Ui, id: Id, transform: PlotTransform) -> bool {
        match self {
            Self::Corner(c) => c.drag(ui, id, transform),
            Self::Smooth(s) => s.drag(ui, id, transform),
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        match self {
            Self::Corner(c) => c.ui(ui),
            Self::Smooth(s) => s.ui(ui),
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
