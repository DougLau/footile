// cubic.rs     Example plotting a cubic bezier arc
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
    p.move_to(8f32, 16f32);
    p.cubic_to(64f32, -16f32, 64f32, 48f32, 0f32, 32f32);
    p.stroke();
    p.write_pgm("./cubic.pgm").unwrap();
}
