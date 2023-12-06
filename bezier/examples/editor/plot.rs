use bezier::{Bezier, CornerPoint, Curve, CurvePoint, Point2D, Segment, Shape, SmoothPoint};
use egui_plot::{Line, PlotPoints, PlotUi, Points};

use crate::{
    configure::{
        self, Configure, CurvePlotConfig, CurvePointPlotConfig, PlotConfig, PointPlotConfig,
        ViewConfig,
    },
    point::Point,
};

fn point(p: &Point, ui: &mut PlotUi, opt: &PointPlotConfig) {
    ui.points(
        Points::new(PlotPoints::Owned(vec![p.0]))
            .shape(opt.mark)
            .filled(true)
            .radius(opt.size as f32 / 2.0)
            .color(opt.color),
    )
}

fn corner_point(
    p: &CornerPoint<Point>, ui: &mut PlotUi, view: &ViewConfig, opt: &CurvePointPlotConfig,
) {
    if view.point {
        point(p.point(), ui, &opt.point);
    }

    if view.ctrl {
        if let Some(c) = p.in_ctrl() {
            point(c, ui, &opt.in_ctrl);
            segment(&Segment::new(*p.point(), *c), ui, &opt.in_handle);
        }
        if let Some(c) = p.out_ctrl() {
            point(c, ui, &opt.out_ctrl);
            segment(&Segment::new(*p.point(), *c), ui, &opt.out_handle);
        }
    }
}

fn smooth_point(
    p: &SmoothPoint<Point>, ui: &mut PlotUi, view: &ViewConfig, opt: &CurvePointPlotConfig,
) {
    if view.point {
        point(p.point(), ui, &opt.point);
    }

    if view.ctrl {
        let in_ctrl = p.in_ctrl();
        let out_ctrl = p.out_ctrl();

        point(&in_ctrl, ui, &opt.in_ctrl);
        point(&out_ctrl, ui, &opt.out_ctrl);

        segment(&Segment::new(*p.point(), in_ctrl), ui, &opt.in_handle);
        segment(&Segment::new(*p.point(), out_ctrl), ui, &opt.out_handle);
    }
}

fn curve_point(p: &CurvePoint<Point>, ui: &mut PlotUi, view: &ViewConfig, opt: &PlotConfig) {
    match p {
        CurvePoint::Corner(c) => corner_point(c, ui, view, &opt.cornel),
        CurvePoint::Smooth(s) => smooth_point(s, ui, view, &opt.smooth),
    }
}

fn segment(segment: &Segment<Point>, ui: &mut PlotUi, opt: &CurvePlotConfig) {
    let f = segment.parametric_function();
    let line = Line::new(PlotPoints::from_parametric_callback(
        move |t| f(t).tuple(),
        0.0..=1.0,
        opt.samples,
    ))
    .color(opt.color)
    .width(opt.width as f32);

    ui.line(line)
}

fn bezier(bezier: &Bezier<Point>, ui: &mut PlotUi, opt: &CurvePlotConfig) {
    let f = bezier.parametric_function();

    let line = Line::new(PlotPoints::from_parametric_callback(
        move |t| f(t).tuple(),
        0.0..=1.0,
        opt.samples,
    ))
    .color(opt.color)
    .width(opt.width as f32);

    ui.line(line);
}

fn curve(c: &Curve<Point>, ui: &mut PlotUi, opt: &PlotConfig) {
    match c {
        Curve::Segment(s) => segment(s, ui, &opt.segment),
        Curve::Bezier(b) => bezier(b, ui, &opt.bezier),
    }
}

pub fn shape(shape: &Shape<Point>, ui: &mut PlotUi, conf: &Configure) {
    if shape.points().is_empty() {
        return;
    }

    for point in shape.points() {
        curve_point(point, ui, &conf.view, &conf.plot)
    }

    if configure::read().view.curve {
        for c in shape.curves() {
            curve(&c, ui, &conf.plot);
        }
    }
}
