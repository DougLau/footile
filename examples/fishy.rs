// fishy.rs     Example fishy drawing
//
// Copyright (c) 2017  Douglas P Lau
//
extern crate footile;

use footile::{FillRule, PlotterBuilder, Raster};

fn main() {
    let mut p = PlotterBuilder::new().width(128).height(128).build();
    let mut r = Raster::new(p.width(), p.height());
    p.pen_width(3f32, false)
     .move_to(112f32, 24f32)
     .line_to(-32f32, 24f32)
     .cubic_to(-96f32, -48f32, -96f32, 80f32, 0f32, 32f32)
     .line_to(32f32, 24f32)
     .line_to(-16f32, -40f32)
     .close()
     .fill(FillRule::EvenOdd);
    r.composite(p.mask(), [127u8, 96u8, 96u8]);
    p.clear()
     .stroke();
    r.composite(p.mask(), [255u8, 208u8, 208u8]);
    p.clear()
     .reset()
     .pen_width(2f32, false)
     .move_to(24f32, 48f32)
     .line_to(8f32, 8f32)
     .move_to(0f32, -8f32)
     .line_to(-8f32, 8f32)
     .stroke();
    r.composite(p.mask(), [0u8, 0u8, 0u8]);
    r.write_png("./fishy.png").unwrap();
}
