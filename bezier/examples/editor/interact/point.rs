use bezier::Point2D;
use eframe::{
    egui::{DragValue, Id, PointerButton, Response, Sense, Ui},
    epaint::{Pos2, Rect, Vec2},
};
use egui_plot::PlotTransform;

use crate::point::Point;

pub struct PointInteract {
    transform: PlotTransform,
    position: Pos2,
    response: Option<Response>,
}

impl PointInteract {
    pub fn new(point: &Point, id: Id, ui: &Ui, transform: PlotTransform, size: f64) -> Self {
        let pp = point.0;

        let position = transform.position_from_point(&pp);

        let mut result = Self {
            transform,
            position,
            response: None,
        };

        let bound = transform.bounds();
        if !bound.is_valid() {
            return result;
        }

        let [x_min, y_min] = bound.min();
        let [x_max, y_max] = bound.max();
        let half = size / 2.0;

        if pp.x + half < x_min || pp.x - half > x_max || pp.y + half < y_min || pp.y - half > y_max
        {
            return result;
        }

        let rect = Rect::from_center_size(result.position, Vec2::splat(size as f32));
        let response = ui.interact(rect, id, Sense::click_and_drag());

        result.response.replace(response);

        result
    }

    pub fn drag(&mut self, p: &mut Point) -> bool {
        if let Some(delta) = self.drag_delta() {
            *p = p.plus(&delta);
            return true;
        }

        false
    }

    pub fn drag_delta(&mut self) -> Option<Point> {
        if let Some(resp) = self.response.as_mut() {
            if resp.dragged_by(PointerButton::Primary) {
                let delta_pos = resp.drag_delta();
                let d = self.transform.dvalue_dpos();
                return Some(Point::from_xy(
                    delta_pos.x as f64 * d[0],
                    delta_pos.y as f64 * d[1],
                ));
            }
        }

        None
    }

    pub fn clicked(&self) -> bool {
        self.response
            .as_ref()
            .map(|r| r.clicked())
            .unwrap_or_default()
    }

    pub fn context_menu(&mut self, add_contents: impl FnOnce(&mut Ui)) {
        if let Some(resp) = self.response.take() {
            self.response.replace(resp.context_menu(add_contents));
        }
    }
}

pub fn controls(p: &mut Point, ui: &mut Ui, text: &str) -> bool {
    let mut changed = false;

    ui.horizontal(|ui| {
        ui.label(text);

        if ui.add(DragValue::new(&mut p.0.x).prefix("x: ")).changed() {
            changed = true;
        }

        if ui.add(DragValue::new(&mut p.0.y).prefix("y: ")).changed() {
            changed = true;
        };
    });

    changed
}
