use bezier::Point2D;
use egui_plot::PlotPoint;

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Point(pub PlotPoint);

impl Point {
    #[inline(always)]
    pub fn new(x: f64, y: f64) -> Self {
        Self(PlotPoint::new(x, y))
    }
}

impl From<PlotPoint> for Point {
    fn from(pp: PlotPoint) -> Self {
        Self(pp)
    }
}

impl From<Point> for PlotPoint {
    fn from(point: Point) -> Self {
        point.0
    }
}

impl Point2D for Point {
    #[inline(always)]
    fn x(&self) -> f64 {
        self.0.x
    }

    #[inline(always)]
    fn y(&self) -> f64 {
        self.0.y
    }

    #[inline(always)]
    fn from_xy(x: f64, y: f64) -> Self {
        Self::new(x, y)
    }
}
