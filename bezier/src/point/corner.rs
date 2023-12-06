use super::Point2D;

/// Control point is free, you can use
/// `{in/out}_ctrl_mut` / `update_{in/out}_ctrl` / `remove_{in/out}_ctrl` to change them as you
/// wish.
///
/// But center point can only be modified by `move_delta` and `move_to` function, which allows
/// the control points follow its movement.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CornerPoint<P> {
    in_ctrl: Option<P>,
    point: P,
    out_ctrl: Option<P>,
}

/// Builder
impl<P> CornerPoint<P> {
    pub fn new(point: P) -> Self {
        Self {
            point,
            in_ctrl: None,
            out_ctrl: None,
        }
    }

    pub fn with_in_ctrl(mut self, point: P) -> Self {
        self.in_ctrl.replace(point);
        self
    }

    pub fn with_out_ctrl(mut self, point: P) -> Self {
        self.out_ctrl.replace(point);
        self
    }
}

/// Getter/setter
impl<P> CornerPoint<P> {
    pub fn point(&self) -> &P {
        &self.point
    }

    pub fn has_in_ctrl(&self) -> bool {
        self.in_ctrl.is_some()
    }

    pub fn in_ctrl(&self) -> Option<&P> {
        self.in_ctrl.as_ref()
    }

    pub fn in_ctrl_mut(&mut self) -> Option<&mut P> {
        self.in_ctrl.as_mut()
    }

    pub fn update_in_ctrl(&mut self, val: P) {
        self.in_ctrl.replace(val);
    }

    pub fn remove_in_ctrl(&mut self) {
        self.in_ctrl.take();
    }

    pub fn has_out_ctrl(&self) -> bool {
        self.out_ctrl.is_some()
    }

    pub fn out_ctrl(&self) -> Option<&P> {
        self.out_ctrl.as_ref()
    }

    pub fn out_ctrl_mut(&mut self) -> Option<&mut P> {
        self.out_ctrl.as_mut()
    }

    pub fn update_out_ctrl(&mut self, val: P) {
        self.out_ctrl.replace(val);
    }

    pub fn remove_out_ctrl(&mut self) {
        self.out_ctrl.take();
    }
}

/// Move
impl<P: Point2D> CornerPoint<P> {
    pub fn move_delta(&mut self, delta: P, move_ctrl: bool) {
        self.point = self.point.plus(&delta);
        if move_ctrl {
            if let Some(m) = self.in_ctrl_mut() {
                *m = m.plus(&delta)
            }
            if let Some(m) = self.out_ctrl_mut() {
                *m = m.plus(&delta)
            }
        }
    }

    pub fn move_to(&mut self, target: P, move_ctrl: bool) {
        let delta = target.minus(&self.point);
        self.move_delta(delta, move_ctrl);
    }

    fn move_ctrl_delta(&self, ctrl: &P, delta_x: f64, delta_y: f64, keep_dir: bool) -> P {
        if keep_dir {
            let dir = ctrl.minus(self.point()).normalize();
            let d = P::from_xy(delta_x, delta_y).dot(&dir);
            ctrl.plus(&dir.scale(d))
        } else {
            ctrl.plus(&P::from_xy(delta_x, delta_y))
        }
    }

    pub fn move_in_ctrl_delta(&mut self, delta_x: f64, delta_y: f64, keep_dir: bool) {
        if let Some(in_ctrl) = self.in_ctrl() {
            let ctrl = self.move_ctrl_delta(in_ctrl, delta_x, delta_y, keep_dir);
            self.update_in_ctrl(ctrl);
        }
    }

    pub fn move_out_ctrl_delta(&mut self, delta_x: f64, delta_y: f64, keep_dir: bool) {
        if let Some(out_ctrl) = self.out_ctrl() {
            let ctrl = self.move_ctrl_delta(out_ctrl, delta_x, delta_y, keep_dir);
            self.update_out_ctrl(ctrl);
        }
    }
}
