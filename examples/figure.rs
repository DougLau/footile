// figure.rs        Example using a figure
//
// Copyright (c) 2017  Douglas P Lau
//
extern crate footile;

use footile::{ FillRule, PlotterBuilder };

fn main() {
    let mut p = PlotterBuilder::new()
                               .width(64)
                               .height(64)
                               .build();
    p.pen_width(1f32, false);
    p.move_to(4f32, 4f32);
    p.line_to(28f32, 12f32);
    p.line_to(28f32, -12f32);
    p.line_to(-12f32, 28f32);
    p.line_to(12f32, 28f32);
    p.line_to(-28f32, -4f32);
    p.line_to(-28f32, 4f32);
    p.line_to(12f32, -28f32);
    p.close();
    p.fill(FillRule::EvenOdd);
    p.write_pgm("./figure.pgm").unwrap();
}
