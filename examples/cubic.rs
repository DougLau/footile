// cubic.rs     Example plotting a cubic bezier arc
//
// Copyright (c) 2017  Douglas P Lau
//
extern crate footile;

use footile::Vec2;
use footile::Plotter;

fn main() {
    let mut p = Plotter::new(64, 64, 0.5f32);
    p.pen_width(2f32, false);
    p.line_to(Vec2::new(8f32, 16f32));
    p.cubic_to(Vec2::new(64f32, -16f32),
               Vec2::new(64f32, 48f32),
               Vec2::new(0f32, 32f32));
    p.rasterize_stroke(true);
    p.get_mask().write_pgm("./cubic.pgm").unwrap();
}
