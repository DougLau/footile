// lib.rs      Footile crate.
//
// Copyright (c) 2017-2018  Douglas P Lau
//
//! Footile is a 2D vector graphics library.  It can be used to fill and stroke
//! paths.  These are created using typical vector drawing primitives such as
//! lines and bézier splines.
//!
extern crate png;

mod imgbuf;
mod geom;
mod gray8;
mod mask;
mod fig;
mod fixed;
mod path;
mod pixel;
mod plotter;
mod raster;
mod rgba8;
mod stroker;

pub use geom::Transform;
pub use gray8::Gray8;
pub use mask::Mask;
pub use path::{FillRule,Path2D,PathBuilder,PathOp};
pub use plotter::Plotter;
pub use raster::Raster;
pub use rgba8::Rgba8;
pub use stroker::JoinStyle;
