// lib.rs      Footile crate.
//
// Copyright (c) 2017  Douglas P Lau
//
//! Footile is a 2D vector graphics library.  It can be used to fill and stroke
//! paths.  These are created using typical vector drawing primitives such as
//! lines and bézier splines.
//!
extern crate png;

mod geom;
mod mask;
mod fig;
mod plotter;

pub use fig::FillRule;
pub use mask::Mask;
pub use plotter::{ JoinStyle, Plotter, PlotterBuilder };
