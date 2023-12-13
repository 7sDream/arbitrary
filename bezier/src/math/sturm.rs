use alloc::vec::Vec;
use core::ops::{Deref, Range};

use super::Poly;

#[derive(Debug, Clone, PartialEq)]
pub struct SturmSeq(Vec<Poly>);

impl Deref for SturmSeq {
    type Target = [Poly];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Sign {
    Zero,
    Positive,
    Negative,
}

enum IsolateTask {
    Check(f64, f64, Option<usize>, Option<usize>),
    Split(f64, f64, usize, usize),
}

enum IsolateTaskResult {
    Discard(usize, usize),
    Return(usize, usize),
    Split(usize, usize),
}

struct IsolateRootState {
    queue: Vec<IsolateTask>,
    eps: f64,
}

impl IsolateRootState {
    fn new(start: f64, end: f64, degree: usize, eps: f64) -> Self {
        let mut queue = Vec::with_capacity(degree + 1);
        queue.push(IsolateTask::Check(start, end, None, None));
        Self { queue, eps }
    }

    fn pop_task(&mut self) -> Option<IsolateTask> {
        self.queue.pop()
    }

    fn add_task(&mut self, task: IsolateTask) {
        self.queue.push(task);
    }
}

impl SturmSeq {
    pub fn new(poly: &Poly) -> Self {
        let mut result = Vec::with_capacity(poly.degree());

        result.push(poly.clone());

        if poly.degree() == 0 {
            return Self(result);
        }

        let mut divided = poly.clone();
        let mut last: Poly = poly.derivative();
        loop {
            let (_, r) = divided.div(&last);
            if r.is_zero() {
                break;
            } else {
                result.push(last.clone());
                divided = last;
                last = r.neg();
            }
        }

        result.push(last);

        Self(result)
    }

    fn eval(&self, x: f64) -> impl Iterator<Item = f64> + '_ {
        self.0.iter().map(move |p| p.eval(x))
    }

    fn signs_at(&self, x: f64) -> impl Iterator<Item = Sign> + '_ {
        self.eval(x).map(|x| {
            if x == 0.0 {
                Sign::Zero
            } else if x.is_sign_negative() {
                Sign::Negative
            } else {
                Sign::Positive
            }
        })
    }

    fn sign_changes_at(&self, x: f64) -> usize {
        let mut changes = 0;

        self.signs_at(x)
            .filter(|s| !matches!(s, Sign::Zero))
            .reduce(|last, current| {
                if current != last {
                    changes += 1
                }
                current
            });

        changes
    }

    pub fn isolate_real_roots(&self, start: f64, end: f64, eps: f64) -> Vec<(f64, f64)> {
        self.isolate_real_roots_iter(start, end, eps).collect()
    }

    // Isolate all **real** roots (zero point) of origin polynomial equation in **left open right
    // closed** interval (start, end]. Yields a series of (f64, f64) tuple, which is also left
    // open right closed, each interval only contains one root. These interval lengths are
    // guaranteed to be smaller than the `eps` parameter.
    //
    // # Panics
    //
    // When start/end is not finite, that is: infinite or NaN.
    pub fn isolate_real_roots_iter(
        &self, start: f64, end: f64, eps: f64,
    ) -> impl Iterator<Item = (f64, f64)> + '_ {
        assert!(start.is_finite());
        assert!(end.is_finite());
        assert!(start <= end);

        let mut state = IsolateRootState::new(start, end, self[0].degree(), eps);
        core::iter::from_fn(move || self.isolate_roots_iter_fn(&mut state))
    }

    fn isolate_roots_iter_fn(&self, state: &mut IsolateRootState) -> Option<(f64, f64)> {
        while let Some(task) = state.pop_task() {
            match task {
                IsolateTask::Check(start, end, s, e) => {
                    let result = self.root_range_check(start, end, s, e, state.eps);

                    match result {
                        IsolateTaskResult::Discard(_, _) => (),
                        IsolateTaskResult::Return(_, _) => return Some((start, end)),
                        IsolateTaskResult::Split(s, e) => {
                            state.add_task(IsolateTask::Split(start, end, s, e));
                        }
                    }
                }
                IsolateTask::Split(start, end, s, e) => {
                    let mid = (start + end) / 2.0;
                    let roots = s - e;

                    let left = self.root_range_check(start, mid, Some(s), None, state.eps);

                    match left {
                        IsolateTaskResult::Discard(_, m) => {
                            state.add_task(IsolateTask::Check(mid, end, Some(m), Some(e)));
                        }
                        IsolateTaskResult::Return(_, m) => {
                            if roots > 1 {
                                state.add_task(IsolateTask::Check(mid, end, Some(m), Some(e)));
                            }
                            return Some((start, mid));
                        }
                        IsolateTaskResult::Split(_, m) => {
                            state.add_task(IsolateTask::Split(start, mid, s, m));
                            if roots > s - m {
                                state.add_task(IsolateTask::Check(mid, end, Some(m), Some(e)));
                            }
                        }
                    }
                }
            }
        }

        None
    }

    fn root_range_check(
        &self, start: f64, end: f64, s: Option<usize>, e: Option<usize>, eps: f64,
    ) -> IsolateTaskResult {
        let s = s.unwrap_or_else(|| self.sign_changes_at(start));
        let e = e.unwrap_or_else(|| self.sign_changes_at(end));

        if e == s {
            return IsolateTaskResult::Discard(s, e);
        }

        if e + 1 == s && start + eps >= end {
            return IsolateTaskResult::Return(s, e);
        }

        IsolateTaskResult::Split(s, e)
    }
}

#[cfg(test)]
mod test {
    use super::super::*;

    #[test]
    fn poly_sturm_seq() {
        let poly: Poly = [1.0, 1.0, 0.0, -1.0, -1.0].into_iter().collect();
        let sturm = SturmSeq::new(&poly);

        assert_eq!(sturm.len(), 5);
        assert_eq!(sturm[0], poly);
        assert_eq!(sturm[1], poly.derivative());
        assert_eq!(sturm[2].coefficients(), [
            3.0 / 16.0,
            3.0 / 4.0,
            15.0 / 16.0
        ]);
        assert_eq!(sturm[3].coefficients(), [-32.0, -64.0]);
        assert_eq!(sturm[4].coefficients(), [-3.0 / 16.0]);

        assert_eq!(sturm.sign_changes_at(f64::NEG_INFINITY), 3);
        assert_eq!(sturm.sign_changes_at(f64::INFINITY), 1);

        dbg!(sturm.isolate_real_roots(-3.0, 6.0, 1e-3));
    }

    #[test]
    fn poly_sturm_play() {
        let poly: Poly = [1.0, -4.0, 2.0, 0.0, -3.0, 7.0].into_iter().collect();
        let sturm = SturmSeq::new(&poly);
        dbg!(sturm.isolate_real_roots(-7.0, 7.0, 1e-6));
    }
}
