// drop.rs      Example plotting a drop
//
// Copyright (c) 2017  Douglas P Lau
//
extern crate footile;

use footile::PlotterBuilder;

fn main() {
    let mut p = PlotterBuilder::new().build();
    p.pen_width(3f32, false);
    p.move_to(50f32, 34f32);
    p.cubic_to(4f32, 16f32, 16f32, 28f32, 0f32, 32f32);
    p.cubic_to(-16f32, -4f32, -4f32, -16f32, 0f32, -32f32);
    p.close();
    p.stroke();
    p.write_png("./drop.png").unwrap();
}
