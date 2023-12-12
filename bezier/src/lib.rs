//! Alley is a vector path/shape library with an API designed primarily for editing in a GUI.
//!
//! ## Concept
//!
//! TODO: concept picture.
//!
//! - [Shape] (or path if you do not close it) consists of [Curve]s, each [Curve] is generated from
//!   two [CurvePoint].
//! - An [CurvePoint] can have an incoming control point and an outgoing control point.
//! - According to the relative relationship between [CurvePoint] and it's two control point,
//!   [CurvePoint] is divided into two kind: [CornerPoint] and [SmoothPoint].
//! - [CornerPoint]: The control point does not necessarily exist. The two control points are
//!   located independently.
//! - [SmoothPoint]: The control point must exist. The two control points and the endpoint itself
//!   are collinear.
//!
//! Therefore, a [Curve] is determined by the following four points: the starting point, the
//! outgoing control point of the starting point, the incoming control point of the ending
//! point, and the ending point.
//!
//! If neither control point exists, then the two [CurvePoint] define a line [Segment]. Otherwise,
//! it is a [Bezier] curve.
//!
//! ## Create
//!
//! You can create [Shape] using the concept above. a iterator of [CurvePoint] can be collected into
//! [Shape].
//!
//! Or, you can use the ShapePainter, to construct a [Shape] in a way more focus on each curve other
//! than each point.
//!
//! Besides, you can parse a SVG path command string into a [Shape] using the
//! [Shape::parse_svg_path].
//!
//! ## Render
//!
//! Alley is designed for editing not rendering, it's out of scope.
//!
//! You can refer to ...

#![cfg_attr(not(test), no_std)]

#[macro_use]
extern crate alloc;

mod curve;
mod point;
mod shape;
mod math;

pub use self::{curve::*, math::*, point::*, shape::*};
