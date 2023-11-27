use eframe::epaint::Color32;
use egui_plot::MarkerShape;

pub struct PointPlotOption {
    pub mark: MarkerShape,
    pub size: f64,
    pub color: Color32,
}

pub struct LinePlotOption {
    pub width: f64,
    pub color: Color32,
    pub samples: usize,
}

pub struct CurvePointPlotOption {
    pub point: PointPlotOption,
    pub in_ctrl: PointPlotOption,
    pub in_ctrl_link: LinePlotOption,
    pub out_ctrl: PointPlotOption,
    pub out_ctrl_link: LinePlotOption,
}

pub const CORNEL_POINT: CurvePointPlotOption = CurvePointPlotOption {
    point: PointPlotOption {
        mark: MarkerShape::Square,
        size: 16.0,
        color: Color32::DARK_GRAY,
    },
    in_ctrl: PointPlotOption {
        mark: MarkerShape::Square,
        size: 12.0,
        color: Color32::DARK_GREEN,
    },
    in_ctrl_link: LinePlotOption {
        width: 1.0,
        color: Color32::DARK_GREEN,
        samples: 2,
    },
    out_ctrl: PointPlotOption {
        mark: MarkerShape::Square,
        size: 12.0,
        color: Color32::DARK_RED,
    },
    out_ctrl_link: LinePlotOption {
        width: 1.0,
        color: Color32::DARK_RED,
        samples: 2,
    },
};

pub const SMOOTH_POINT: CurvePointPlotOption = CurvePointPlotOption {
    point: PointPlotOption {
        mark: MarkerShape::Circle,
        size: 16.0,
        color: Color32::GOLD,
    },
    in_ctrl: PointPlotOption {
        mark: MarkerShape::Circle,
        size: 12.0,
        color: Color32::DARK_GREEN,
    },
    in_ctrl_link: LinePlotOption {
        width: 1.0,
        color: Color32::DARK_GREEN,
        samples: 2,
    },
    out_ctrl: PointPlotOption {
        mark: MarkerShape::Circle,
        size: 12.0,
        color: Color32::DARK_RED,
    },
    out_ctrl_link: LinePlotOption {
        width: 1.0,
        color: Color32::DARK_RED,
        samples: 2,
    },
};

pub const BEZIER_CURVE: LinePlotOption = LinePlotOption {
    width: 2.0,
    color: Color32::BLUE,
    samples: 64,
};

pub const LINE_CURVE: LinePlotOption = LinePlotOption {
    width: 2.0,
    color: Color32::BLUE,
    samples: 2,
};
