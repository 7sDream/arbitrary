use bezier::Shape;
use eframe::egui::Ui;

use crate::{controls, point::Point};

impl_window! {
    ShapeDataWindow<Shape<Point>> as "Shape Data" : ShapeWindowState { }
}

impl ShapeDataWindow {
    fn controls(&mut self, ui: &mut Ui, data: &mut Shape<Point>) {
        controls::shape(data, ui, self.id.with("inner"));
    }
}
