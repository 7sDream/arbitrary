use bezier::SmoothPoint;
use eframe::egui::{Id, Ui};
use egui_plot::PlotTransform;

use super::{point::PointInteract, PointAction};
use crate::{
    configure::{CurvePointPlotConfig, ViewConfig},
    controls,
    point::Point,
};

pub struct SmoothPointInteract<'a>(&'a mut SmoothPoint<Point>);

impl<'a> SmoothPointInteract<'a> {
    pub fn new(sp: &'a mut SmoothPoint<Point>) -> Self {
        Self(sp)
    }

    fn ctrl_interact(
        &mut self, ui: &mut Ui, id: Id, transform: &PlotTransform, opt: &CurvePointPlotConfig,
    ) {
        let mut in_act = PointInteract::new(
            &self.0.in_ctrl(),
            id.with("in"),
            ui,
            *transform,
            opt.in_ctrl.size,
        );
        if let Some(delta) = in_act.drag_delta() {
            // TODO: keep dir drag when press some key
            self.0.move_in_ctrl_delta(delta.0.x, delta.0.y, false);
        }
        in_act.context_menu(|ui| {
            controls::smooth_point_theta(self.0, ui);
            controls::smooth_point_in_length(self.0, ui);

            if ui.button("Same length as out").clicked() {
                self.0.update_in_length(self.0.out_length());
                ui.close_menu();
            }
        });

        let mut out_act = PointInteract::new(
            &self.0.out_ctrl(),
            id.with("out"),
            ui,
            *transform,
            opt.out_ctrl.size,
        );
        if let Some(delta) = out_act.drag_delta() {
            self.0.move_out_ctrl_delta(delta.0.x, delta.0.y, false);
        }
        out_act.context_menu(|ui| {
            controls::smooth_point_theta(self.0, ui);
            controls::smooth_point_out_length(self.0, ui);

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
            controls::smooth_point(self.0, ui);

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
        &mut self, ui: &mut Ui, id: Id, transform: &PlotTransform, view: &ViewConfig,
        opt: &CurvePointPlotConfig,
    ) -> Option<PointAction> {
        let mut action = None;

        if view.point {
            action = self.point_interact(ui, id, transform, opt);
        }

        if view.ctrl {
            self.ctrl_interact(ui, id, transform, opt);
        }

        action
    }
}
