use bezier::{CornerPoint, Point2D};
use eframe::egui::{Id, Ui};
use egui_plot::PlotTransform;

use super::{point::PointInteract, PointAction};
use crate::{
    configure::{self, CurvePointPlotConfig},
    controls,
    point::Point,
};

pub struct CornerPointInteract<'a>(&'a mut CornerPoint<Point>);

impl<'a> CornerPointInteract<'a> {
    pub fn new(cp: &'a mut CornerPoint<Point>) -> Self {
        Self(cp)
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

        if let Some(delta) = act.drag_delta() {
            self.0.move_delta(delta, true);
        }

        act.context_menu(|ui| {
            controls::corner_point(self.0, ui);

            if !self.0.has_in_ctrl() || !self.0.has_out_ctrl() {
                ui.menu_button("Add", |ui| {
                    ui.add_enabled_ui(!self.0.has_in_ctrl(), |ui| {
                        if ui.button("In ctrl point").clicked() {
                            let p = Point::from_xy(self.0.point().x() - 10.0, self.0.point().y());
                            self.0.update_in_ctrl(p);
                            ui.close_menu();
                        }
                    });
                    ui.add_enabled_ui(!self.0.has_out_ctrl(), |ui| {
                        if ui.button("Out ctrl point").clicked() {
                            let p = Point::from_xy(self.0.point().x() + 10.0, self.0.point().y());
                            self.0.update_out_ctrl(p);
                            ui.close_menu();
                        }
                    });
                });
            }

            ui.menu_button("Delete", |ui| {
                ui.add_enabled_ui(self.0.in_ctrl().is_some(), |ui| {
                    if ui.button("In ctrl point").clicked() {
                        self.0.remove_in_ctrl();
                        ui.close_menu();
                    }
                });

                ui.add_enabled_ui(self.0.out_ctrl().is_some(), |ui| {
                    if ui.button("Out ctrl point").clicked() {
                        self.0.remove_out_ctrl();
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

        if act.clicked() {
            action.replace(PointAction::Click);
        }

        action
    }

    fn ctrl_interact(
        &mut self, ui: &mut Ui, id: Id, transform: &PlotTransform, opt: &CurvePointPlotConfig,
    ) {
        let mut delete_in = false;
        if let Some(p) = self.0.in_ctrl_mut() {
            let mut in_act = PointInteract::new(p, id.with("in"), ui, *transform, opt.in_ctrl.size);
            in_act.drag(p);
            in_act.context_menu(|ui| {
                controls::point(p, ui, "In ctrl");

                if ui.button("Delete").clicked() {
                    delete_in = true;
                    ui.close_menu();
                }
            });
        }
        if delete_in {
            self.0.remove_in_ctrl();
        }

        let mut delete_out = false;
        if let Some(p) = self.0.out_ctrl_mut() {
            let mut out_act =
                PointInteract::new(p, id.with("out"), ui, *transform, opt.out_ctrl.size);
            out_act.drag(p);
            out_act.context_menu(|ui| {
                controls::point(p, ui, "Out ctrl");

                if ui.button("Delete").clicked() {
                    delete_out = true;
                    ui.close_menu();
                }
            });
        }
        if delete_out {
            self.0.remove_out_ctrl();
        }
    }

    pub fn interact(
        &mut self, ui: &mut Ui, id: Id, transform: &PlotTransform,
    ) -> Option<PointAction> {
        let conf = configure::read();
        let opt = &conf.plot.cornel;

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
