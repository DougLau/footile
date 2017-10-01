// quad.rs     Example plotting a quadratic bezier arc
//
// Copyright (c) 2017  Douglas P Lau
//
extern crate footile;
use footile::Vec2;
use footile::Plotter;

fn main() {
    let mut p = Plotter::new(64, 64, 0.5f32);
    p.pen_width(2f32, false);
    p.line_to(Vec2::new(0f32, 16f32));
    p.quad_to(Vec2::new(128f32, 16f32), Vec2::new(0f32, 32f32));
    p.rasterize_stroke(true);
    let ref m = p.get_mask();
    m.write_pgm("./quad.pgm").unwrap();
}
