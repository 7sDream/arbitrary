use eframe::epaint::Color32;
use egui_plot::MarkerShape;

pub const CORNEL_MARK: MarkerShape = MarkerShape::Square;
pub const SMOOTH_MARK: MarkerShape = MarkerShape::Circle;
pub const CTRL_MARK: MarkerShape = MarkerShape::Circle;

pub const MAIN_POINT_COLOR: Color32 = Color32::DARK_GREEN;
pub const CTRL_1_COLOR: Color32 = Color32::DARK_RED;
pub const CTRL_2_COLOR: Color32 = Color32::DARK_RED;

pub const CTRL_LINK_LINE_COLOR: Color32 = Color32::BROWN;
pub const CURVE_COLOR: Color32 = Color32::BLUE;
