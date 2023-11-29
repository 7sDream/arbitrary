use core::cmp::Ordering;

use super::{point::PointExt, Bezier, Point, Segment};

pub struct Nearest {
    pub index: usize,
    pub t: f64,
    pub point: Point,
    pub distance: f64,
}

impl PartialEq for Nearest {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Eq for Nearest {}

impl PartialOrd for Nearest {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Nearest {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.distance.total_cmp(&other.distance) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.index.cmp(&other.index) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.t.total_cmp(&other.t) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.point.0.total_cmp(&other.point.0) {
            Ordering::Equal => {}
            ord => return ord,
        }
        self.point.1.total_cmp(&other.point.1)
    }
}

impl Nearest {
    pub fn new_from_point(point: &Point, target: &Point) -> Self {
        let distance = point.minus(target).length_from_origin();
        Self {
            index: 0,
            t: 0.0,
            point: *point,
            distance,
        }
    }

    pub fn new_from_segment(segment: &Segment, t: f64, target: &Point) -> Self {
        let point = segment.at(t);
        let distance = point.minus(target).length_from_origin();
        Self {
            index: 0,
            t,
            point,
            distance,
        }
    }

    pub fn new_from_bezier(line: &Bezier, t: f64, target: &Point) -> Self {
        let point = line.at(t);
        let distance = point.minus(target).length_from_origin();
        Self {
            index: 0,
            t,
            point,
            distance,
        }
    }

    pub fn with_index(mut self, val: usize) -> Self {
        self.index = val;
        self
    }
}
