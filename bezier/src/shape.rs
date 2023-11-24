use eframe::egui::{CollapsingHeader, Id, Ui};
use egui_plot::{PlotTransform, PlotUi};

use crate::{bezier::Bezier, constants::CURVE_COLOR, line::LineSegment, point::CurvePoint};

pub struct Shape {
    points: Vec<CurvePoint>,
    close: bool,
}

impl Shape {
    pub fn empty() -> Self {
        Self {
            points: vec![],
            close: true,
        }
    }

    pub fn push(&mut self, point: impl Into<CurvePoint>) {
        self.points.push(point.into());
    }

    pub fn insert(&mut self, index: usize, point: impl Into<CurvePoint>) {
        self.points.insert(index, point.into());
    }

    pub fn toggle_close(&mut self) {
        self.close = !self.close;
    }

    fn plot_segment(plot: &mut PlotUi, start: &CurvePoint, end: &CurvePoint) {
        let sp = start.point();
        let ep = end.point();

        match (start.out_ctrl(), end.in_ctrl()) {
            (Some(ctrl1), Some(ctrl2)) => {
                Bezier::new(sp, ctrl1.as_ref(), ctrl2.as_ref(), ep).plot(plot, CURVE_COLOR, 2.0);
            }
            (Some(ctrl), None) | (None, Some(ctrl)) => {
                Bezier::new_quad(sp, ctrl.as_ref(), ep).plot(plot, CURVE_COLOR, 2.0);
            }
            (None, None) => {
                LineSegment::new(sp, ep).plot(plot, CURVE_COLOR, 2.0);
            }
        }
    }

    pub fn plot(&self, plot: &mut PlotUi) {
        if self.points.is_empty() {
            return;
        }

        for point in &self.points {
            point.plot(plot);
        }

        for curve in self.points.windows(2) {
            let [start, end] = curve else {
                unreachable!("we use windows api here so curve must have 2 item")
            };
            Self::plot_segment(plot, start, end);
        }

        if self.close && self.points.len() >= 2 {
            Self::plot_segment(
                plot,
                self.points.last().unwrap(),
                self.points.first().unwrap(),
            );
        }
    }

    pub fn interact(&mut self, ui: &mut Ui, id: Id, transform: PlotTransform) -> bool {
        for (i, point) in self.points.iter_mut().enumerate() {
            if point.drag(ui, id.with(i), transform) {
                return true;
            }
        }

        false
    }

    pub fn ui(&mut self, ui: &mut Ui, id: Id) {
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
