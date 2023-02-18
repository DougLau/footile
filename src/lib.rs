// lib.rs      Footile crate.
//
// Copyright (c) 2017-2021  Douglas P Lau
//
//! Footile is a 2D vector graphics library.  It can be used to fill and stroke
//! paths.  These are created using typical vector drawing primitives such as
//! lines and bézier splines.
//!
//! ## Example
//! ```rust
//! use footile::{FillRule, Path2D, Plotter};
//! use pix::{matte::Matte8, Raster};
//!
//! let fish = Path2D::default()
//!     .relative()
//!     .pen_width(3.0)
//!     .move_to(112.0, 24.0)
//!     .line_to(-32.0, 24.0)
//!     .cubic_to(-96.0, -48.0, -96.0, 80.0, 0.0, 32.0)
//!     .line_to(32.0, 24.0)
//!     .line_to(-16.0, -40.0)
//!     .close()
//!     .finish();
//! let raster = Raster::with_clear(128, 128);
//! let mut p = Plotter::new(raster);
//! p.fill(FillRule::NonZero, &fish, Matte8::new(255));
//! ```
#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]

mod fig;
mod fixed;
mod geom;
mod imgbuf;
mod path;
mod plotter;
mod stroker;
mod vid;

pub use path::{FillRule, Path2D, PathOp, TransformOp};
pub use plotter::Plotter;
pub use stroker::JoinStyle;
