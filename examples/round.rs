// round.rs     Example plotting a rounded stroke
//
// Copyright (c) 2017  Douglas P Lau
//
extern crate footile;

use footile::{PlotterBuilder, JoinStyle};

fn main() {
    let mut p = PlotterBuilder::new().build();
    p.pen_width(40f32, false);
    p.join_style(JoinStyle::Round);
    p.move_to(10f32, 60f32);
    p.line_to(50f32, 0f32);
    p.line_to(0f32, -50f32);
    p.stroke();
    p.write_png("./round.png").unwrap();
}
