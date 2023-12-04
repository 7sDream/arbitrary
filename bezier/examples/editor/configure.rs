use std::sync::OnceLock;

use eframe::epaint::{
    mutex::{RwLock, RwLockReadGuard, RwLockWriteGuard},
    Color32,
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

pub const DEFAULT_CORNEL_POINT_PLOT_CONFIG: CurvePointPlotConfig = CurvePointPlotConfig {
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

pub const DEFAULT_SMOOTH_POINT_PLOT_CONFIG: CurvePointPlotConfig = CurvePointPlotConfig {
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

pub const DEFAULT_CURVE_SEGMENT_PLOT_CONFIG: CurvePlotConfig = CurvePlotConfig {
    width: 2.0,
    color: Color32::BLUE,
    samples: 64,
};

pub const DEFAULT_CURVE_BEZIER_PLOT_CONFIG: CurvePlotConfig = CurvePlotConfig {
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
            cornel: DEFAULT_CORNEL_POINT_PLOT_CONFIG.clone(),
            smooth: DEFAULT_SMOOTH_POINT_PLOT_CONFIG.clone(),
            segment: DEFAULT_CURVE_BEZIER_PLOT_CONFIG.clone(),
            bezier: DEFAULT_CURVE_SEGMENT_PLOT_CONFIG.clone(),
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
    pub grid: bool,
    pub point: bool,
    pub ctrl: bool,
    pub curve: bool,
}

impl Default for ViewConfig {
    fn default() -> Self {
        Self {
            grid: true,
            point: true,
            ctrl: true,
            curve: true,
        }
    }
}

#[derive(Default)]
pub struct Configure {
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
