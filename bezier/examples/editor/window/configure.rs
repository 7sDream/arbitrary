use eframe::{
    egui::{ComboBox, Grid, Id, Layout, Slider, Ui},
    emath::Align,
};
use egui_plot::MarkerShape;

use super::{FloatWindow, WindowState};
use crate::configure::{
    self, Configure, CurvePlotConfig, CurvePointPlotConfig, PlotConfig, PointPlotConfig, ViewConfig,
};

trait MarkerShapeName {
    fn name(&self) -> &'static str;
}

impl MarkerShapeName for MarkerShape {
    fn name(&self) -> &'static str {
        match self {
            MarkerShape::Circle => "o | circle",
            MarkerShape::Diamond => "<> | diamond",
            MarkerShape::Square => "[] | square",
            MarkerShape::Cross => "x | cross",
            MarkerShape::Plus => "+ | plus",
            MarkerShape::Up => "^ | up",
            MarkerShape::Down => "v | down",
            MarkerShape::Left => "< | left",
            MarkerShape::Right => "> | right",
            MarkerShape::Asterisk => "* | asterisk",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum ConfigureWindowTab {
    View,
    Plot,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum PlotViewTab {
    Corner,
    Smooth,
    Curve,
}

impl_window! {
    ConfigureWindow as "Configure" : ConfigureWindowState {
        tab: ConfigureWindowTab = ConfigureWindowTab::View,
        plot_tab: PlotViewTab = PlotViewTab::Corner,
    }
}

impl ConfigureWindow {
    pub fn tab_view(ui: &mut Ui, conf: &mut ViewConfig) {
        ui.vertical(|ui| {
            ui.checkbox(&mut conf.grid, "Grid");
            ui.checkbox(&mut conf.point, "Point");
            ui.checkbox(&mut conf.ctrl, "Control point");
            ui.checkbox(&mut conf.curve, "Curve");
        });
    }

    fn table_title(ui: &mut Ui, text: &str) {
        ui.vertical_centered(|ui| {
            ui.label(text);
        });
    }

    fn point_marker_combo_box(ui: &mut Ui, id: Id, marker: &mut MarkerShape) {
        ComboBox::from_id_source(id)
            .selected_text(marker.name())
            .show_ui(ui, |ui| {
                ui.selectable_value(marker, MarkerShape::Circle, MarkerShape::Circle.name());
                ui.selectable_value(marker, MarkerShape::Diamond, MarkerShape::Diamond.name());
                ui.selectable_value(marker, MarkerShape::Square, MarkerShape::Square.name());
                ui.selectable_value(marker, MarkerShape::Cross, MarkerShape::Cross.name());
                ui.selectable_value(marker, MarkerShape::Plus, MarkerShape::Plus.name());
                ui.selectable_value(marker, MarkerShape::Up, MarkerShape::Up.name());
                ui.selectable_value(marker, MarkerShape::Down, MarkerShape::Down.name());
                ui.selectable_value(marker, MarkerShape::Left, MarkerShape::Left.name());
                ui.selectable_value(marker, MarkerShape::Right, MarkerShape::Right.name());
                ui.selectable_value(marker, MarkerShape::Asterisk, MarkerShape::Asterisk.name());
            });
    }

    fn point_row(ui: &mut Ui, id: Id, text: &str, conf: &mut PointPlotConfig) {
        ui.label(text);
        ui.add(Slider::new(&mut conf.size, 0.0..=32.0));
        ui.vertical_centered(|ui| {
            ui.color_edit_button_srgba(&mut conf.color);
        });
        Self::point_marker_combo_box(ui, id.with("marker"), &mut conf.mark);
        ui.end_row();
    }

    fn curve_row(ui: &mut Ui, _id: Id, text: &str, conf: &mut CurvePlotConfig) {
        ui.label(text);
        ui.add(Slider::new(&mut conf.width, 0.0..=32.0));
        ui.vertical_centered(|ui| {
            ui.color_edit_button_srgba(&mut conf.color);
        });
        ui.add(Slider::new(&mut conf.samples, 2..=256));
        ui.end_row();
    }

    fn point_table<F>(ui: &mut Ui, id: Id, content: F)
    where
        F: FnOnce(&mut Ui),
    {
        Grid::new(id)
            .num_columns(4)
            .min_col_width(64.0)
            .show(ui, |ui| {
                ui.label("");
                ui.vertical_centered(|ui| {
                    ui.label("Size");
                });
                ui.vertical_centered(|ui| {
                    ui.label("Color");
                });
                ui.vertical_centered(|ui| {
                    ui.label("Shape");
                });
                ui.end_row();
                content(ui);
            });
    }

    fn curve_table<F>(ui: &mut Ui, id: Id, content: F)
    where
        F: FnOnce(&mut Ui),
    {
        Grid::new(id)
            .num_columns(4)
            .min_col_width(64.0)
            .show(ui, |ui| {
                ui.label("");
                ui.vertical_centered(|ui| {
                    ui.label("Width");
                });
                ui.vertical_centered(|ui| {
                    ui.label("Color");
                });
                ui.vertical_centered(|ui| {
                    ui.label("Samples");
                });
                ui.end_row();
                content(ui);
            });
    }

    fn tab_plot_point(
        &mut self, ui: &mut Ui, id: Id, conf: &mut CurvePointPlotConfig, def: &CurvePointPlotConfig,
    ) {
        ui.group(|ui| {
            Self::table_title(ui, "Control point");
            ui.add_space(8.0);
            Self::point_table(ui, id.with("point-grid"), |ui| {
                Self::point_row(ui, id.with("point"), "Main", &mut conf.point);
                Self::point_row(ui, id.with("in-point"), "In", &mut conf.in_ctrl);
                Self::point_row(ui, id.with("out-point"), "Out", &mut conf.out_ctrl);
            });
        });

        ui.add_space(8.0);

        ui.group(|ui| {
            Self::table_title(ui, "Handle");
            ui.add_space(8.0);
            Self::curve_table(ui, id.with("handle-grid"), |ui| {
                Self::curve_row(ui, id.with("in-handle"), "In", &mut conf.in_handle);
                Self::curve_row(ui, id.with("out-handle"), "Out", &mut conf.out_handle);
            });
        });

        ui.add_space(8.0);

        ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
            if ui.button("Reset to default").clicked() {
                *conf = def.clone();
            }
        });
    }

    fn tab_plot_curve(&mut self, ui: &mut Ui, id: Id, conf: &mut PlotConfig) {
        ui.group(|ui| {
            Self::curve_table(ui, id.with("curve-grid"), |ui| {
                Self::curve_row(ui, id.with("segment-curve"), "Segment", &mut conf.segment);
                Self::curve_row(ui, id.with("bezier-curve"), "Bezier", &mut conf.bezier);
            });
        });

        ui.add_space(8.0);

        ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
            if ui.button("Reset to default").clicked() {
                conf.segment = configure::DEFAULT_CURVE_BEZIER_PLOT_CONFIG.clone();
                conf.bezier = configure::DEFAULT_CURVE_SEGMENT_PLOT_CONFIG.clone();
            }
        });
    }

    fn tab_plot(&mut self, ui: &mut Ui, conf: &mut PlotConfig) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.state.plot_tab, PlotViewTab::Corner, "Corner");
            ui.selectable_value(&mut self.state.plot_tab, PlotViewTab::Smooth, "Smooth");
            ui.selectable_value(&mut self.state.plot_tab, PlotViewTab::Curve, "Curve");
        });
        ui.separator();

        match self.state.plot_tab {
            PlotViewTab::Corner => {
                self.tab_plot_point(
                    ui,
                    self.id.with("corner"),
                    &mut conf.cornel,
                    &configure::DEFAULT_CORNEL_POINT_PLOT_CONFIG,
                );
            }
            PlotViewTab::Smooth => {
                self.tab_plot_point(
                    ui,
                    self.id.with("smooth"),
                    &mut conf.smooth,
                    &configure::DEFAULT_SMOOTH_POINT_PLOT_CONFIG,
                );
            }
            PlotViewTab::Curve => {
                self.tab_plot_curve(ui, self.id.with("curve"), conf);
            }
        };
    }

    fn controls(&mut self, ui: &mut Ui) {
        let conf: &mut Configure = &mut configure::write();

        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.state.tab, ConfigureWindowTab::View, "View");
            ui.selectable_value(&mut self.state.tab, ConfigureWindowTab::Plot, "Plot");
        });
        ui.separator();
        match self.state.tab {
            ConfigureWindowTab::View => Self::tab_view(ui, &mut conf.view),
            ConfigureWindowTab::Plot => self.tab_plot(ui, &mut conf.plot),
        }
    }
}
