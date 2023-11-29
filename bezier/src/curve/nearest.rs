use core::cmp::Ordering;

use super::{Bezier, Point2D, Segment};

pub struct Nearest<P: Point2D> {
    pub index: usize,
    pub t: f64,
    pub point: P,
    pub distance: f64,
}

impl<P: Point2D> PartialEq for Nearest<P> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl<P: Point2D> Eq for Nearest<P> {}

impl<P: Point2D> PartialOrd for Nearest<P> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<P: Point2D> Ord for Nearest<P> {
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
        self.point.total_cmp(&other.point)
    }
}

impl<P: Point2D> Nearest<P> {
    pub fn new_from_point(point: &P, target: &P) -> Self {
        let distance = point.minus(target).length_from_origin();
        Self {
            index: 0,
            t: 0.0,
            point: point.clone(),
            distance,
        }
    }

    pub fn new_from_segment(segment: &Segment<P>, t: f64, target: &P) -> Self {
        let point = segment.at(t);
        let distance = point.minus(target).length_from_origin();
        Self {
            index: 0,
            t,
            point,
            distance,
        }
    }

    pub fn new_from_bezier(line: &Bezier<P>, t: f64, target: &P) -> Self {
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
