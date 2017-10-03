// quad.rs     Example plotting a quadratic bezier arc
//
// Copyright (c) 2017  Douglas P Lau
//
extern crate footile;

use footile::PlotterBuilder;

fn main() {
    let mut p = PlotterBuilder::new()
                               .width(64)
                               .height(64)
                               .build();
    p.pen_width(2f32, false);
    p.move_to(0f32, 16f32);
    p.quad_to(128f32, 16f32, 0f32, 32f32);
    p.stroke();
    p.write_pgm("./quad.pgm").unwrap();
}
