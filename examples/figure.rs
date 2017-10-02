// figure.rs        Example using a figure
//
// Copyright (c) 2017  Douglas P Lau
//
extern crate footile;

use footile::{ FillRule, Plotter, Vec2 };

fn main() {
    let mut p = Plotter::new(64, 64, 0.5f32);
    p.pen_width(1f32, false);
    p.line_to(Vec2::new(4f32, 4f32));
    p.line_to(Vec2::new(28f32, 12f32));
    p.line_to(Vec2::new(28f32, -12f32));
    p.line_to(Vec2::new(-12f32, 28f32));
    p.line_to(Vec2::new(12f32, 28f32));
    p.line_to(Vec2::new(-28f32, -4f32));
    p.line_to(Vec2::new(-28f32, 4f32));
    p.line_to(Vec2::new(12f32, -28f32));
    p.close(true);
    p.rasterize_fill(FillRule::EvenOdd, true);
    p.get_mask().write_pgm("./figure.pgm").unwrap();
}
