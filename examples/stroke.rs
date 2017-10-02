// stroke.rs        Example stroking a figure
//
// Copyright (c) 2017  Douglas P Lau
//
extern crate footile;

use footile::{ Plotter, Vec2 };

fn main() {
    let mut p = Plotter::new(64, 64, 0.5f32);
    p.pen_width(5f32, false);
    p.line_to(Vec2::new(16f32, 48f32));
    p.line_to(Vec2::new(32f32, 0f32));
    p.line_to(Vec2::new(-16f32, -32f32));
    p.close(true);
    p.rasterize_stroke(true);
    p.get_mask().write_pgm("./stroke.pgm").unwrap();
}
