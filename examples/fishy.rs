// fishy.rs
extern crate footile;

use footile::{Color,FillRule,PathBuilder,Plotter,Raster};

fn main() {
    let fish = PathBuilder::new().relative().pen_width(3f32)
                           .move_to(112f32, 24f32)
                           .line_to(-32f32, 24f32)
                           .cubic_to(-96f32, -48f32, -96f32, 80f32, 0f32, 32f32)
                           .line_to(32f32, 24f32)
                           .line_to(-16f32, -40f32)
                           .close().build();
    let eye = PathBuilder::new().relative().pen_width(2f32)
                          .move_to(24f32, 48f32)
                          .line_to(8f32, 8f32)
                          .move_to(0f32, -8f32)
                          .line_to(-8f32, 8f32)
                          .build();
    let mut p = Plotter::new(128, 128);
    let mut r = Raster::new(p.width(), p.height());
    p.fill(&fish, FillRule::NonZero);
    r.composite(p.mask(), Color::rgb(127, 96, 96));
    p.clear().stroke(&fish);
    r.composite(p.mask(), Color::rgb(255, 208, 208));
    p.clear().stroke(&eye);
    r.composite(p.mask(), Color::rgb(0, 0, 0));
    r.write_png("./fishy.png").unwrap();
}
