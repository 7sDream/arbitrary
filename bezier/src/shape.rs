use eframe::epaint::Vec2;
use egui_plot::{PlotPoint, PlotUi};

use crate::{
    bezier::{self, Cubic},
    line::Line,
    point::{CornerPoint, CurvePoint, SmoothPoint},
};

pub struct Shape {
    points: Vec<CurvePoint>,
    close: bool,
}

impl Shape {
    pub fn empty() -> Self {
        Self {
            points: vec![],
            close: false,
        }
    }

    pub fn push(&mut self, point: impl Into<CurvePoint>) {
        self.points.push(point.into());
    }

    pub fn insert(&mut self, index: usize, point: impl Into<CurvePoint>) {
        self.points.insert(index, point.into());
    }

    pub fn plot(&self, plot: &mut PlotUi) {
        if self.points.is_empty() {
            return;
        }

        for curve in self.points.windows(2) {
            let [start, end] = curve else {
                unreachable!("we use windows api here so curve must have 2 item")
            };

            let sp = start.point();
            let ep = end.point();

            let ctrls = match (start.out_ctrl(), end.in_ctrl()) {
                (Some(ctrl1), Some(ctrl2)) => Some((ctrl1, ctrl2)),
                (Some(ctrl), None) | (None, Some(ctrl)) => {
                    Some(bezier::quad_to_cubic_ctrl(&sp, &ctrl, &ep))
                }
                (None, None) => None,
            };

            match ctrls {
                Some((ctrl1, ctrl2)) => {
                    plot.line(Cubic::new(sp, ep, ctrl1, ctrl2).curve());
                }
                None => {
                    plot.line(Line::new(sp, ep).curve());
                }
            }
        }
    }
}

impl FromIterator<CurvePoint> for Shape {
    fn from_iter<T: IntoIterator<Item = CurvePoint>>(iter: T) -> Self {
        Self {
            points: iter.into_iter().collect(),
            close: false,
        }
    }
}
