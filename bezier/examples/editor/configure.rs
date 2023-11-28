use std::sync::OnceLock;

use eframe::{
    egui::{Button, CollapsingHeader, ComboBox, Context, Grid, Id, Key, Slider, Ui, Window},
    epaint::{
        mutex::{RwLock, RwLockReadGuard, RwLockWriteGuard},
        Color32,
    },
};
use egui_plot::MarkerShape;

#[derive(Clone)]
pub struct PointPlotConfig {
    pub mark: MarkerShape,
    pub size: f64,
    pub color: Color32,
}

#[derive(Clone)]
pub struct CurvePlotConfig {
    pub width: f64,
    pub color: Color32,
    pub samples: usize,
}

#[derive(Clone)]
pub struct CurvePointPlotConfig {
    pub point: PointPlotConfig,
    pub in_ctrl: PointPlotConfig,
    pub in_handle: CurvePlotConfig,
    pub out_ctrl: PointPlotConfig,
    pub out_handle: CurvePlotConfig,
}

const CORNEL_POINT: CurvePointPlotConfig = CurvePointPlotConfig {
    point: PointPlotConfig {
        mark: MarkerShape::Square,
        size: 16.0,
        color: Color32::DARK_GRAY,
    },
    in_ctrl: PointPlotConfig {
        mark: MarkerShape::Square,
        size: 12.0,
        color: Color32::DARK_GREEN,
    },
    in_handle: CurvePlotConfig {
        width: 1.0,
        color: Color32::DARK_GREEN,
        samples: 2,
    },
    out_ctrl: PointPlotConfig {
        mark: MarkerShape::Square,
        size: 12.0,
        color: Color32::DARK_RED,
    },
    out_handle: CurvePlotConfig {
        width: 1.0,
        color: Color32::DARK_RED,
        samples: 2,
    },
};

const SMOOTH_POINT: CurvePointPlotConfig = CurvePointPlotConfig {
    point: PointPlotConfig {
        mark: MarkerShape::Circle,
        size: 16.0,
        color: Color32::GOLD,
    },
    in_ctrl: PointPlotConfig {
        mark: MarkerShape::Circle,
        size: 12.0,
        color: Color32::DARK_GREEN,
    },
    in_handle: CurvePlotConfig {
        width: 1.0,
        color: Color32::DARK_GREEN,
        samples: 2,
    },
    out_ctrl: PointPlotConfig {
        mark: MarkerShape::Circle,
        size: 12.0,
        color: Color32::DARK_RED,
    },
    out_handle: CurvePlotConfig {
        width: 1.0,
        color: Color32::DARK_RED,
        samples: 2,
    },
};

const CURVE_BEZIER: CurvePlotConfig = CurvePlotConfig {
    width: 2.0,
    color: Color32::BLUE,
    samples: 64,
};

const CURVE_SEGMENT: CurvePlotConfig = CurvePlotConfig {
    width: 2.0,
    color: Color32::BLUE,
    samples: 2,
};

#[derive(Clone)]
pub struct PlotConfig {
    pub cornel: CurvePointPlotConfig,
    pub smooth: CurvePointPlotConfig,
    pub segment: CurvePlotConfig,
    pub bezier: CurvePlotConfig,
}

impl Default for PlotConfig {
    fn default() -> Self {
        Self {
            cornel: CORNEL_POINT.clone(),
            smooth: SMOOTH_POINT.clone(),
            segment: CURVE_SEGMENT.clone(),
            bezier: CURVE_BEZIER.clone(),
        }
    }
}

pub struct SubWindowConfig {
    pub configure: bool,
    pub shape_data: bool,
}

impl Default for SubWindowConfig {
    fn default() -> Self {
        Self {
            configure: true,
            shape_data: false,
        }
    }
}

pub struct ViewConfig {
    pub show_ctrl: bool,
}

impl Default for ViewConfig {
    fn default() -> Self {
        Self { show_ctrl: true }
    }
}

#[derive(Default)]
pub struct Configure {
    pub windows: SubWindowConfig,
    pub view: ViewConfig,
    pub plot: PlotConfig,
}

static CONF: OnceLock<RwLock<Configure>> = OnceLock::new();

pub fn read() -> RwLockReadGuard<'static, Configure> {
    CONF.get_or_init(RwLock::default).read()
}

pub fn write() -> RwLockWriteGuard<'static, Configure> {
    CONF.get_or_init(RwLock::default).write()
}

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
            MarkerShape::Left => "<| | left",
            MarkerShape::Right => "|> | right",
            MarkerShape::Asterisk => "* | asterisk",
        }
    }
}

fn controls_windows_config(option: &mut SubWindowConfig, _id: Id, ui: &mut Ui) {
    ui.horizontal(|ui| {
        let setting = ui
            .add(Button::new("Configure").selected(option.configure))
            .on_hover_text_at_pointer("Press S to reopen this configure window.");

        if setting.clicked() {
            option.configure = !option.configure;
        }

        if ui
            .add(Button::new("Shape data").selected(option.shape_data))
            .clicked()
        {
            option.shape_data = !option.shape_data;
        }
    });
}

fn controls_view_config(option: &mut ViewConfig, _id: Id, ui: &mut Ui) {
    ui.checkbox(&mut option.show_ctrl, "Show ctrl point and handle");
}

fn controls_mark_shape(marker: &mut MarkerShape, id: Id, ui: &mut Ui) {
    ComboBox::from_id_source(id.with("child"))
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

fn controls_point_plot_option(option: &mut PointPlotConfig, text: &str, id: Id, ui: &mut Ui) {
    CollapsingHeader::new(text)
        .id_source(id.with("header"))
        .show(ui, |ui| {
            Grid::new(id.with("grid")).show(ui, |ui| {
                ui.label("Size: ");
                ui.add(Slider::new(&mut option.size, 0.0..=32.0));
                ui.end_row();

                ui.label("Marker: ");
                controls_mark_shape(&mut option.mark, id.with("marker"), ui);
                ui.end_row();

                ui.label("Color: ");
                ui.color_edit_button_srgba(&mut option.color);
                ui.end_row();
            })
        });
}

fn controls_curve_plot_config(option: &mut CurvePlotConfig, text: &str, id: Id, ui: &mut Ui) {
    CollapsingHeader::new(text)
        .id_source(id.with("header"))
        .show(ui, |ui| {
            Grid::new(id.with("grid")).show(ui, |ui| {
                ui.label("Width: ");
                ui.add(Slider::new(&mut option.width, 0.0..=32.0));
                ui.end_row();

                ui.label("Color: ");
                ui.color_edit_button_srgba(&mut option.color);
                ui.end_row();

                ui.label("Samples: ");
                ui.add(Slider::new(&mut option.samples, 2..=256));
                ui.end_row();
            })
        });
}

fn controls_curve_point_plot_config(
    option: &mut CurvePointPlotConfig, text: &str, id: Id, ui: &mut Ui,
) {
    CollapsingHeader::new(text).id_source(id).show(ui, |ui| {
        controls_point_plot_option(&mut option.point, "Point", id.with("point"), ui);
        controls_point_plot_option(&mut option.in_ctrl, "In ctrl point", id.with("in_ctrl"), ui);
        controls_curve_plot_config(
            &mut option.in_handle,
            "In ctrl handle",
            id.with("in_handle"),
            ui,
        );
        controls_point_plot_option(
            &mut option.out_ctrl,
            "Out ctrl point",
            id.with("out_ctrl"),
            ui,
        );
        controls_curve_plot_config(
            &mut option.out_handle,
            "Out ctrl handle",
            id.with("out_handle"),
            ui,
        );
    });
}

fn controls_plot_config(option: &mut PlotConfig, id: Id, ui: &mut Ui) {
    controls_curve_point_plot_config(&mut option.cornel, "Cornel point", id.with("cornel"), ui);
    controls_curve_point_plot_config(&mut option.smooth, "Smooth point", id.with("smooth"), ui);
    controls_curve_plot_config(&mut option.segment, "Segment", id.with("segment"), ui);
    controls_curve_plot_config(&mut option.bezier, "Bezier", id.with("bezier"), ui);
}

pub fn controls_configure(config: &mut Configure, id: Id, ui: &mut Ui) {
    controls_windows_config(&mut config.windows, id.with("windows"), ui);
    controls_view_config(&mut config.view, id.with("view"), ui);
    controls_plot_config(&mut config.plot, id.with("plot"), ui);
}

pub fn configure_window(ctx: &Context) {
    let configure: &mut Configure = &mut write();

    if !configure.windows.configure && ctx.input(|i| i.key_pressed(Key::S)) {
        configure.windows.configure = true;
    }

    if configure.windows.configure {
        Window::new("Configure")
            .id(Id::new("configure_window"))
            .auto_sized()
            .default_open(true)
            .show(ctx, |ui| {
                controls_configure(configure, Id::new("configure"), ui)
            });
    }
}
