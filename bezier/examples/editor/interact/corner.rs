use bezier::{CornerPoint, Point2D};
use eframe::egui::{Id, Ui};
use egui_plot::PlotTransform;

use super::{point::PointInteract, PointAction};
use crate::{
    configure::{CurvePointPlotConfig, ViewConfig},
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
                ui.add_enabled_ui(self.0.has_in_ctrl(), |ui| {
                    if ui.button("In ctrl point").clicked() {
                        self.0.remove_in_ctrl();
                        ui.close_menu();
                    }
                });

                ui.add_enabled_ui(self.0.has_out_ctrl(), |ui| {
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
        if self.0.has_in_ctrl() {
            let mut in_act = PointInteract::new(
                self.0.in_ctrl().unwrap(),
                id.with("in"),
                ui,
                *transform,
                opt.in_ctrl.size,
            );
            if let Some(delta) = in_act.drag_delta() {
                self.0.move_in_ctrl_delta(delta.0.x, delta.0.y, false);
            }
            in_act.context_menu(|ui| {
                controls::point(self.0.in_ctrl_mut().unwrap(), ui, "In ctrl");

                if ui.button("Delete").clicked() {
                    self.0.remove_in_ctrl();
                    ui.close_menu();
                }
            });
        }

        if self.0.has_out_ctrl() {
            let mut in_act = PointInteract::new(
                self.0.out_ctrl().unwrap(),
                id.with("in"),
                ui,
                *transform,
                opt.in_ctrl.size,
            );
            if let Some(delta) = in_act.drag_delta() {
                self.0.move_out_ctrl_delta(delta.0.x, delta.0.y, false);
            }
            in_act.context_menu(|ui| {
                controls::point(self.0.out_ctrl_mut().unwrap(), ui, "Out ctrl");

                if ui.button("Delete").clicked() {
                    self.0.remove_out_ctrl();
                    ui.close_menu();
                }
            });
        }
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
