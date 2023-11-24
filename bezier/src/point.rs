use eframe::{
    egui::{DragValue, Id, PointerButton, Sense, Ui},
    epaint::{Color32, Rect, Vec2},
};
use egui_plot::{MarkerShape, PlotPoint, PlotPoints, PlotTransform, Points};

const POINT_SIZE: f32 = 8.0;
const POINT_DRAG_RADIUS: f32 = 8.0;

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
}

pub struct SmoothPoint {
    in_ctrl: PlotPoint,
    point: PlotPoint,
    out_ctrl: PlotPoint,
}

pub enum CurvePoint {
    Corner(CornerPoint),
    Smooth(SmoothPoint),
}

impl CurvePoint {
    pub fn point(&self) -> PlotPoint {
        match self {
            Self::Corner(c) => c.point,
            Self::Smooth(s) => s.point,
        }
    }

    pub fn in_ctrl(&self) -> Option<PlotPoint> {
        match self {
            Self::Corner(c) => c.in_ctrl,
            Self::Smooth(s) => Some(s.out_ctrl),
        }
    }

    pub fn out_ctrl(&self) -> Option<PlotPoint> {
        match self {
            Self::Corner(c) => c.out_ctrl,
            Self::Smooth(s) => Some(s.out_ctrl),
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

pub fn to_drawable(p: PlotPoint, color: Color32) -> Points {
    Points::new(PlotPoints::Owned(vec![p]))
        .shape(MarkerShape::Circle)
        .filled(true)
        .color(color)
        .radius(POINT_SIZE)
}

pub fn ui(text: &str, p: &mut PlotPoint, ui: &mut Ui) {
    ui.label(text);
    ui.vertical(|ui| {
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

pub struct DraggablePoint<'a>(pub &'a mut PlotPoint);

impl<'a> DraggablePoint<'a> {
    pub fn drag(&mut self, id: Id, ui: &mut Ui, transform: PlotTransform) {
        let drag_rect_size = Vec2::splat(POINT_DRAG_RADIUS);
        let center = transform.position_from_point(self.0);
        let drag_rect = Rect::from_center_size(center, drag_rect_size);
        let resp = ui.interact(drag_rect, id, Sense::drag());

        if resp.dragged_by(PointerButton::Primary) {
            let delta = resp.drag_delta();
            let sp = transform.position_from_point(self.0) + delta;
            *self.0 = transform.value_from_position(sp);
        }
    }
}
