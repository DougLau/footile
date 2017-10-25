// lib.rs      Footile crate.
//
// Copyright (c) 2017  Douglas P Lau
//
//! Footile is a 2D vector graphics library.  It can be used to fill and stroke
//! paths.  These are created using typical vector drawing primitives such as
//! lines and bézier splines.
//!
extern crate libc;
extern crate palette;
extern crate png;

mod imgbuf;
mod geom;
mod mask;
mod fig;
mod path;
mod plotter;
mod raster;

pub use geom::Transform;
pub use mask::Mask;
pub use path::{FillRule, JoinStyle, Path2D, PathBuilder};
pub use plotter::Plotter;
pub use raster::Raster;
