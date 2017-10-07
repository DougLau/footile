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
    p.pen_width(2f32, false)
     .move_to(0f32, 16f32)
     .quad_to(128f32, 16f32, 0f32, 32f32)
     .stroke();
    p.mask().write_png("./quad.png").unwrap();
}
