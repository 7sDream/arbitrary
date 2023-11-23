use eframe::{
    egui::{Id, Sense, Ui},
    epaint::{Color32, Rect, Vec2},
};
use egui_plot::{Line, MarkerShape, PlotPoint, PlotPoints, PlotTransform, Points};

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
const END_COLOR: Color32 = Color32::DARK_RED;
const CTRL_1_COLOR: Color32 = Color32::DARK_BLUE;
const CTRL_2_COLOR: Color32 = Color32::DARK_BLUE;

const BOUND_COLOR: Color32 = Color32::BROWN;
const CURVE_COLOR: Color32 = Color32::BLUE;

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

    fn points(&self) -> impl Iterator<Item = Points> {
        [self.start, self.ctrl1, self.ctrl2, self.end]
            .into_iter()
            .zip([START_COLOR, CTRL_1_COLOR, CTRL_2_COLOR, END_COLOR])
            .map(Self::point)
    }

    fn polygon(&self) -> Line {
        Line::new(PlotPoints::Owned(vec![
            self.start, self.end, self.ctrl2, self.ctrl1, self.start,
        ]))
        .color(BOUND_COLOR)
        .width(1.0)
    }

    fn parametric_function(&self) -> impl Fn(f64) -> (f64, f64) {
        let start = self.start;
        let end = self.end;
        let ctrl1 = self.ctrl1;
        let ctrl2 = self.ctrl2;

        move |t| {
            let nt = 1.0 - t;
            let x = start.x * nt.powi(3)
                + 3.0 * ctrl1.x * t * nt.powi(2)
                + 3.0 * ctrl2.x * t.powi(2) * nt
                + end.x * t.powi(3);
            let y = start.y * nt.powi(3)
                + 3.0 * ctrl1.y * t * nt.powi(2)
                + 3.0 * ctrl2.y * t.powi(2) * nt
                + end.y * t.powi(3);
            (x, y)
        }
    }

    fn curve(&self) -> Line {
        Line::new(PlotPoints::from_parametric_callback(
            self.parametric_function(),
            0.0..=1.0,
            64,
        ))
        .color(CURVE_COLOR)
        .width(2.0)
    }

    pub fn ui(&mut self, id: Id, ui: &mut Ui) {
        let transform = egui_plot::Plot::new(id)
            .data_aspect(1.0)
            .include_x(-45)
            .include_x(45)
            .include_y(-60)
            .include_y(20)
            .y_axis_width(3)
            .show_x(false)
            .show_y(false)
            .show(ui, |plot| {
                plot.line(self.polygon());
                for point in self.points() {
                    plot.points(point);
                }
                plot.line(self.curve());
            })
            .transform;

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
