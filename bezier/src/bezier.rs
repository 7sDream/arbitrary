use eframe::egui::{Id, Ui};
use egui_plot::{Line, Plot, PlotPoint, PlotPoints, PlotTransform, PlotUi, Points};

use crate::{
    color::{BOUND_COLOR, CTRL_1_COLOR, CTRL_2_COLOR, CURVE_COLOR, END_COLOR, START_COLOR},
    point::{self, DraggablePoint},
};

pub fn quad_to_cubic_ctrl(
    start: &PlotPoint, ctrl: &PlotPoint, end: &PlotPoint,
) -> (PlotPoint, PlotPoint) {
    fn calc(a: &PlotPoint, b: &PlotPoint) -> PlotPoint {
        let x = a.x + 2.0 * (b.x - a.x) / 3.0;
        let y = a.y + 2.0 * (b.y - a.y) / 3.0;
        PlotPoint { x, y }
    }

    (calc(start, ctrl), calc(end, ctrl))
}

#[derive(Clone)]
pub struct Cubic {
    pub start: PlotPoint,
    pub end: PlotPoint,
    pub ctrl1: PlotPoint,
    pub ctrl2: PlotPoint,
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

    fn points(&self) -> impl Iterator<Item = Points> {
        [self.start, self.ctrl1, self.ctrl2, self.end]
            .into_iter()
            .zip([START_COLOR, CTRL_1_COLOR, CTRL_2_COLOR, END_COLOR])
            .map(|(p, c)| point::to_drawable(p, c))
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

    pub fn curve(&self) -> Line {
        Line::new(PlotPoints::from_parametric_callback(
            self.parametric_function(),
            0.0..=1.0,
            64,
        ))
        .color(CURVE_COLOR)
        .width(2.0)
    }

    pub fn ui(&mut self, id: Id, ui: &mut Ui) {
        ui.push_id(id.with("controls"), |ui| {
            ui.horizontal(|ui| {
                point::ui("Start: ", &mut self.start, ui);
                ui.add_space(16.0);
                point::ui("Ctrl 1: ", &mut self.ctrl1, ui);
                ui.add_space(16.0);
                point::ui("Ctrl 2: ", &mut self.ctrl2, ui);
                ui.add_space(16.0);
                point::ui("End: ", &mut self.end, ui);
            });
        });
    }

    pub fn plot(&mut self, plot: &mut PlotUi, draw_bound: bool) {
        if draw_bound {
            plot.line(self.polygon());
        }
        for point in self.points() {
            plot.points(point);
        }
        plot.line(self.curve());
    }

    pub fn drag(&mut self, transform: PlotTransform, id: Id, ui: &mut Ui) {
        [
            &mut self.start,
            &mut self.ctrl1,
            &mut self.ctrl2,
            &mut self.end,
        ]
        .into_iter()
        .map(DraggablePoint)
        .enumerate()
        .for_each(|(i, mut p)| p.drag(id.with(i), ui, transform));
    }
}
