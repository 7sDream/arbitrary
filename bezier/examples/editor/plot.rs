use bezier::{Bezier, CornerPoint, Curve, CurvePoint, Point, Segment, Shape, SmoothPoint};
use egui_plot::{Line, PlotPoint, PlotPoints, PlotUi, Points};

use crate::configure::{self, CurvePlotConfig, PointPlotConfig};

pub fn plot_point(p: &Point, ui: &mut PlotUi, opt: &PointPlotConfig) {
    ui.points(
        Points::new(PlotPoints::Owned(vec![PlotPoint::new(p.0, p.1)]))
            .shape(opt.mark)
            .filled(true)
            .radius(opt.size as f32 / 2.0)
            .color(opt.color),
    )
}

pub fn plot_corner_point(p: &CornerPoint, ui: &mut PlotUi) {
    let conf = &configure::read();
    let opt = &conf.plot.cornel;

    plot_point(p.point(), ui, &opt.point);

    if conf.view.show_ctrl {
        if let Some(c) = p.in_ctrl() {
            plot_point(c, ui, &opt.in_ctrl);
            plot_segment(&Segment::new(*p.point(), *c), ui, &opt.in_handle);
        }
        if let Some(c) = p.out_ctrl() {
            plot_point(c, ui, &opt.out_ctrl);
            plot_segment(&Segment::new(*p.point(), *c), ui, &opt.out_handle);
        }
    }
}

pub fn plot_smooth_point(p: &SmoothPoint, ui: &mut PlotUi) {
    let conf = &configure::read();
    let opt = &conf.plot.smooth;

    plot_point(p.point(), ui, &opt.point);

    if conf.view.show_ctrl {
        let in_ctrl = p.in_ctrl();
        let out_ctrl = p.out_ctrl();

        plot_point(&in_ctrl, ui, &opt.in_ctrl);
        plot_point(&out_ctrl, ui, &opt.out_ctrl);

        plot_segment(&Segment::new(*p.point(), in_ctrl), ui, &opt.in_handle);
        plot_segment(&Segment::new(*p.point(), out_ctrl), ui, &opt.out_handle);
    }
}

fn plot_curve_point(p: &CurvePoint, ui: &mut PlotUi) {
    match p {
        CurvePoint::Corner(c) => plot_corner_point(c, ui),
        CurvePoint::Smooth(s) => plot_smooth_point(s, ui),
    }
}

pub fn plot_segment(segment: &Segment, ui: &mut PlotUi, opt: &CurvePlotConfig) {
    let line = Line::new(PlotPoints::from_parametric_callback(
        segment.parametric_function(),
        0.0..=1.0,
        opt.samples,
    ))
    .color(opt.color)
    .width(opt.width as f32);

    ui.line(line)
}

pub fn plot_bezier(bezier: &Bezier, ui: &mut PlotUi, opt: &CurvePlotConfig) {
    let line = Line::new(PlotPoints::from_parametric_callback(
        bezier.parametric_function(),
        0.0..=1.0,
        opt.samples,
    ))
    .color(opt.color)
    .width(opt.width as f32);

    ui.line(line);
}

fn plot_curve(c: &Curve, ui: &mut PlotUi) {
    let opt = &configure::read().plot;

    match c {
        Curve::Segment(s) => plot_segment(s, ui, &opt.segment),
        Curve::Bezier(b) => plot_bezier(b, ui, &opt.bezier),
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
