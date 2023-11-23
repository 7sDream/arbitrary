use eframe::{
    egui::{Id, Sense, Ui},
    epaint::{Color32, Rect, Vec2},
};
use egui_plot::{MarkerShape, PlotPoint, PlotPoints, PlotTransform, Points};

#[derive(Clone)]
pub struct Cubic {
    pub start: PlotPoint,
    pub end: PlotPoint,
    pub ctrl1: PlotPoint,
    pub ctrl2: PlotPoint,
}

const POINT_SIZE: f32 = 8.0;
const POINT_DRAG_RADIUS: f32 = 8.0;

const START_COLOR: Color32 = Color32::DARK_GREEN;
const END_COLOR: Color32 = Color32::RED;
const CTRL_1_COLOR: Color32 = Color32::BLUE;
const CTRL_2_COLOR: Color32 = Color32::GOLD;

struct MovablePoint<'a>(&'a mut PlotPoint);

impl<'a> MovablePoint<'a> {
    fn ui(&mut self, id: Id, ui: &mut Ui, transform: PlotTransform) {
        let drag_rect_size = Vec2::splat(POINT_DRAG_RADIUS);
        let center = transform.position_from_point(self.0);
        let drag_rect = Rect::from_center_size(center, drag_rect_size);
        let resp = ui.interact(drag_rect, id, Sense::drag());

        if resp.dragged() {
            let delta = resp.drag_delta();
            let sp = transform.position_from_point(self.0) + delta;
            *self.0 = transform.value_from_position(sp);
        }
    }
}

impl Cubic {
    pub fn new(start: PlotPoint, end: PlotPoint, ctrl1: PlotPoint, ctrl2: PlotPoint) -> Self {
        Self {
            start,
            end,
            ctrl1,
            ctrl2,
        }
    }

    fn point((point, color): (PlotPoint, Color32)) -> Points {
        Points::new(PlotPoints::Owned(vec![point]))
            .shape(MarkerShape::Circle)
            .filled(true)
            .color(color)
            .radius(POINT_SIZE)
    }

    pub fn points(&mut self) -> impl Iterator<Item = Points> {
        [self.start, self.ctrl1, self.ctrl2, self.end]
            .into_iter()
            .zip([START_COLOR, CTRL_1_COLOR, CTRL_2_COLOR, END_COLOR])
            .map(Self::point)
    }

    pub fn ui(&mut self, id: Id, ui: &mut Ui, transform: PlotTransform) {
        [
            &mut self.start,
            &mut self.ctrl1,
            &mut self.ctrl2,
            &mut self.end,
        ]
        .into_iter()
        .map(MovablePoint)
        .enumerate()
        .for_each(|(i, mut p)| p.ui(id.with(i), ui, transform));
    }
}
