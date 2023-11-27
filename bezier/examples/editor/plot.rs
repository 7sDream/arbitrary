use bezier::{Bezier, CornerPoint, Curve, CurvePoint, Point, Segment, Shape, SmoothPoint};
use egui_plot::{Line, PlotPoint, PlotPoints, PlotUi, Points};

use crate::option::{
    LinePlotOption, PointPlotOption, BEZIER_CURVE, CORNEL_POINT, LINE_CURVE, SMOOTH_POINT,
};

pub fn plot_point(p: &Point, ui: &mut PlotUi, opt: PointPlotOption) {
    ui.points(
        Points::new(PlotPoints::Owned(vec![PlotPoint::new(p.0, p.1)]))
            .shape(opt.mark)
            .filled(true)
            .radius(opt.size as f32 / 2.0)
            .color(opt.color),
    )
}

pub fn plot_corner_point(p: &CornerPoint, ui: &mut PlotUi) {
    plot_point(p.point(), ui, CORNEL_POINT.point);
    if let Some(c) = p.in_ctrl() {
        plot_point(c, ui, CORNEL_POINT.in_ctrl);
        plot_segment(&Segment::new(*p.point(), *c), ui, CORNEL_POINT.in_ctrl_link);
    }
    if let Some(c) = p.out_ctrl() {
        plot_point(c, ui, CORNEL_POINT.out_ctrl);
        plot_segment(
            &Segment::new(*p.point(), *c),
            ui,
            CORNEL_POINT.out_ctrl_link,
        );
    }
}

pub fn plot_smooth_point(p: &SmoothPoint, ui: &mut PlotUi) {
    plot_point(p.point(), ui, SMOOTH_POINT.point);

    let in_ctrl = p.in_ctrl();
    let out_ctrl = p.out_ctrl();

    plot_point(&in_ctrl, ui, SMOOTH_POINT.in_ctrl);
    plot_point(&out_ctrl, ui, SMOOTH_POINT.out_ctrl);

    plot_segment(
        &Segment::new(*p.point(), in_ctrl),
        ui,
        SMOOTH_POINT.in_ctrl_link,
    );
    plot_segment(
        &Segment::new(*p.point(), out_ctrl),
        ui,
        SMOOTH_POINT.out_ctrl_link,
    );
}

fn plot_curve_point(p: &CurvePoint, ui: &mut PlotUi) {
    match p {
        CurvePoint::Corner(c) => plot_corner_point(c, ui),
        CurvePoint::Smooth(s) => plot_smooth_point(s, ui),
    }
}

pub fn plot_segment(segment: &Segment, ui: &mut PlotUi, opt: LinePlotOption) {
    let line = egui_plot::Line::new(PlotPoints::from_parametric_callback(
        segment.parametric_function(),
        0.0..=1.0,
        2,
    ))
    .color(opt.color)
    .width(opt.width as f32);

    ui.line(line)
}

pub fn plot_bezier(bezier: &Bezier, ui: &mut PlotUi, opt: LinePlotOption) {
    let line = Line::new(PlotPoints::from_parametric_callback(
        bezier.parametric_function(),
        0.0..=1.0,
        64,
    ))
    .color(opt.color)
    .width(opt.width as f32);

    ui.line(line);
}

fn plot_curve(c: &Curve, ui: &mut PlotUi) {
    match c {
        Curve::Bezier(b) => plot_bezier(b, ui, BEZIER_CURVE),
        Curve::Segment(s) => plot_segment(s, ui, LINE_CURVE),
    }
}

pub fn plot_shape(shape: &Shape, ui: &mut PlotUi) {
    if shape.points().is_empty() {
        return;
    }

    for point in shape.points() {
        plot_curve_point(point, ui)
    }

    for curve in shape.curves() {
        plot_curve(&curve, ui);
    }
}
