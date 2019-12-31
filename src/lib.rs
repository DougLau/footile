// lib.rs      Footile crate.
//
// Copyright (c) 2017-2019  Douglas P Lau
//
//! Footile is a 2D vector graphics library.  It can be used to fill and stroke
//! paths.  These are created using typical vector drawing primitives such as
//! lines and bézier splines.
//!
mod imgbuf;
mod geom;
mod fig;
mod fixed;
mod path;
mod plotter;
mod stroker;

pub use geom::Transform;
pub use path::{FillRule, Path2D, PathBuilder, PathOp};
pub use plotter::Plotter;
pub use stroker::JoinStyle;
