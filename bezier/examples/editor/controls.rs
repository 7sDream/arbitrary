use bezier::{CornerPoint, CurvePoint, Shape, SmoothPoint};
use eframe::egui::{CollapsingHeader, DragValue, Id, Slider, Ui};

use crate::point::Point;

pub fn point(p: &mut Point, ui: &mut Ui, text: &str) -> bool {
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

pub fn corner_point(p: &mut CornerPoint<Point>, ui: &mut Ui) {
    let mut current = *p.point();
    if point(&mut current, ui, "Point") {
        p.move_to(current, true);
    }

    if let Some(p) = p.in_ctrl_mut() {
        point(p, ui, "In ctrl");
    }
    if let Some(p) = p.out_ctrl_mut() {
        point(p, ui, "Out ctrl");
    }
}

pub fn smooth_point_main_point(sp: &mut SmoothPoint<Point>, ui: &mut Ui) {
    point(sp.point_mut(), ui, "Point");
}

pub fn smooth_point_theta(sp: &mut SmoothPoint<Point>, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label("Theta: ");

        let mut theta = sp.out_theta();
        let slider = Slider::new(&mut theta, 0.0..=359.999)
            .smart_aim(true)
            .suffix("°");

        if ui.add(slider).changed() {
            sp.update_out_theta(theta);
        }
    });
}

pub fn smooth_point_in_length(sp: &mut SmoothPoint<Point>, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label("In ctrl: ");

        let mut l = sp.in_length();
        let slider = Slider::new(&mut l, 0.0..=100.0)
            .smart_aim(true)
            .clamp_to_range(false);

        if ui.add(slider).changed() {
            sp.update_in_length(l);
        }
    });
}

pub fn smooth_point_out_length(sp: &mut SmoothPoint<Point>, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label("Out ctrl: ");

        let mut l = sp.out_length();
        let slider = Slider::new(&mut l, 0.0..=100.0)
            .smart_aim(true)
            .clamp_to_range(false);

        if ui.add(slider).changed() {
            sp.update_out_length(l);
        }
    });
}

pub fn smooth_point(sp: &mut SmoothPoint<Point>, ui: &mut Ui) {
    smooth_point_main_point(sp, ui);
    smooth_point_theta(sp, ui);
    smooth_point_in_length(sp, ui);
    smooth_point_out_length(sp, ui);
}

pub fn curve_point(p: &mut CurvePoint<Point>, ui: &mut Ui) {
    match p {
        CurvePoint::Corner(cp) => corner_point(cp, ui),
        CurvePoint::Smooth(sp) => smooth_point(sp, ui),
    }
}

pub fn shape(s: &mut Shape<Point>, ui: &mut Ui, id: Id) {
    let mut deleted = None;
    for (i, p) in s.points_mut().iter_mut().enumerate() {
        if let Some(Some(del)) = CollapsingHeader::new(i.to_string().as_str())
            .id_source(id.with(i))
            .show(ui, |ui| {
                curve_point(p, ui);

                if ui.button("Delete").clicked() {
                    return Some(i);
                }

                None
            })
            .body_returned
        {
            deleted.replace(del);
        }
    }

    if let Some(del) = deleted {
        s.remove(del);
    }
}
