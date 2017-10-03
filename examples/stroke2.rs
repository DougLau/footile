// stroke.rs        Example stroking a figure
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
    p.pen_width(6f32, false);
    p.move_to(16f32, 15f32);
    p.line_to(32f32, 1f32);
    p.line_to(-32f32, 1f32);
    p.line_to(32f32, 15f32);
    p.line_to(-32f32, 15f32);
    p.line_to(32f32, 1f32);
    p.line_to(-32f32, 1f32);
    p.stroke();
    p.write_pgm("./stroke2.pgm").unwrap();
}
