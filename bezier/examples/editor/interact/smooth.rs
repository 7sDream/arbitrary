use bezier::SmoothPoint;
use eframe::egui::{Id, Ui};
use egui_plot::PlotTransform;

use super::{point::PointInteract, PointAction};
use crate::{
    configure::{self, CurvePointPlotConfig},
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
        let mut in_ctrl = self.0.in_ctrl();
        let mut in_act =
            PointInteract::new(&in_ctrl, id.with("in"), ui, *transform, opt.in_ctrl.size);
        if in_act.drag(&mut in_ctrl) {
            self.0.move_in_ctrl_to(&in_ctrl);
        }
        in_act.context_menu(|ui| {
            controls::smooth_point_theta(self.0, ui);
            controls::smooth_point_in_length(self.0, ui);

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
        &mut self, ui: &mut Ui, id: Id, transform: &PlotTransform,
    ) -> Option<PointAction> {
        let conf = configure::read();
        let opt = &conf.plot.smooth;

        let mut action = None;

        if conf.view.point {
            action = self.point_interact(ui, id, transform, opt);
        }

        if conf.view.ctrl {
            self.ctrl_interact(ui, id, transform, opt);
        }

        action
    }
}
