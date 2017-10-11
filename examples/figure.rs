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
    p.move_to(4f32, 4f32)
     .line_to(28f32, 12f32)
     .line_to(28f32, -12f32)
     .line_to(-12f32, 28f32)
     .line_to(12f32, 28f32)
     .line_to(-28f32, -4f32)
     .line_to(-28f32, 4f32)
     .line_to(12f32, -28f32)
     .close()
     .fill(FillRule::EvenOdd);
    p.mask().write_png("./figure.png").unwrap();
}
