// lib.rs      Footile crate.
//
// Copyright (c) 2017-2020  Douglas P Lau
//
//! Footile is a 2D vector graphics library.  It can be used to fill and stroke
//! paths.  These are created using typical vector drawing primitives such as
//! lines and bézier splines.
//!
#![warn(missing_docs)]
#![warn(missing_doc_code_examples)]

mod fig;
mod fixed;
mod geom;
mod imgbuf;
mod path;
mod plotter;
mod stroker;
mod vid;

pub use geom::{Pt, Transform};
pub use path::{FillRule, Path2D, PathOp};
pub use plotter::Plotter;
pub use stroker::JoinStyle;
