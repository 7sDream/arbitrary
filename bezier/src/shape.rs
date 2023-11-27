use crate::{
    curve::{Curve, CurvePoint, Nearest, Point},
    CornerPoint, SmoothPoint,
};

#[derive(Default)]
pub struct Shape {
    points: Vec<CurvePoint>,
    close: bool,
}

impl Shape {
    pub fn closed(&self) -> bool {
        self.close
    }

    pub fn set_close(&mut self, val: bool) {
        self.close = val;
    }

    pub fn toggle_close(&mut self) {
        self.close = !self.close;
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn points(&self) -> &[CurvePoint] {
        &self.points
    }

    pub fn points_mut(&mut self) -> &mut [CurvePoint] {
        &mut self.points
    }

    pub fn with_points<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Vec<CurvePoint>),
    {
        f(&mut self.points)
    }

    pub fn push(&mut self, point: CurvePoint) {
        self.points.push(point);
    }

    pub fn insert(&mut self, index: usize, point: CurvePoint) {
        self.points.insert(index, point);
    }

    pub fn insert_on_curve(&mut self, index: usize, t: f64) {
        assert!((0.0..=1.0).contains(&t));

        let curve = self.curves().nth(index).unwrap();
        let target = curve.at(t);

        match curve {
            Curve::Bezier(b) => {
                let l = self.points.len();

                // split the curve at t
                let (left, right) = b.split_at(t);

                // adjust around point's ctrl points
                self.points[index].update_out_ctrl(left.ctrl1);
                self.points[(index + 1) % l].update_in_ctrl(right.ctrl2);

                // create new point
                let mut p = SmoothPoint::horizontal(target, 1.0, 1.0);
                p.move_in_ctrl_to(&left.ctrl2);
                p.move_out_ctrl_to(&right.ctrl1);

                // insert
                self.insert(index + 1, p.into());
            }
            Curve::Segment(_) => self.insert(index + 1, CornerPoint::new(target).into()),
        }
    }

    pub fn remove(&mut self, index: usize) {
        self.points.remove(index);
    }

    pub fn replace(&mut self, index: usize, point: CurvePoint) {
        self.points[index] = point;
    }

    pub fn curves(&self) -> impl Iterator<Item = Curve> + '_ {
        let mut close_returned = false;

        self.points
            .windows(2)
            .map(|curve| Curve::new(&curve[0], &curve[1]))
            .chain(std::iter::from_fn(move || {
                if !close_returned && self.close && self.points.len() >= 2 {
                    close_returned = true;
                    Some(Curve::new(
                        self.points.last().unwrap(),
                        self.points.first().unwrap(),
                    ))
                } else {
                    None
                }
            }))
    }

    fn nearest_endpoints_iter<'out, 'a: 'out, 'b: 'out>(
        &'a self, target: &'b Point,
    ) -> impl Iterator<Item = Nearest> + 'out {
        self.points
            .iter()
            .enumerate()
            .map(|(i, p)| Nearest::new_from_point(p.point(), target).with_index(i))
    }

    // TODO: support bounding box clip
    pub fn nearest_endpoint(&self, target: &Point) -> Option<Nearest> {
        self.nearest_endpoints_iter(target).min()
    }

    // TODO: support bounding box clip
    pub fn nearest_point_on_curves(&self, target: &Point, allow_endpoint: bool) -> Option<Nearest> {
        let p = self
            .curves()
            .enumerate()
            .flat_map(|(i, s)| s.nearest_to(target, false).map(|p| p.with_index(i)));

        if allow_endpoint {
            p.chain(self.nearest_endpoints_iter(target)).min()
        } else {
            p.min()
        }
    }
}

impl FromIterator<CurvePoint> for Shape {
    fn from_iter<T: IntoIterator<Item = CurvePoint>>(iter: T) -> Self {
        Self {
            points: iter.into_iter().collect(),
            close: true,
        }
    }
}
