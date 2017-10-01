// quad.rs     Example plotting a quadratic bezier arc
//
// Copyright (c) 2017  Douglas P Lau
//
extern crate footile;
use std::fs::File;
use std::io::BufWriter;
use footile::geom::Vec2;
use footile::plotter::Plotter;

fn main() {
    let mut p = Plotter::new(64, 64, 0.5f32);
    p.pen_width(2f32, false);
    p.line_to(Vec2::new(0f32, 16f32));
    p.quad_to(Vec2::new(128f32, 16f32), Vec2::new(0f32, 32f32));
    p.rasterize_stroke(true);
    let ref mut m = p.get_mask();
    let fl = File::create("./quad.pgm").unwrap();
    let mut bw = BufWriter::new(fl);
    let _ = m.write_pgm(bw.get_mut());
}
