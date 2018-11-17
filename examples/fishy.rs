// fishy.rs
extern crate footile;

use footile::{FillRule,PathBuilder,Plotter,Rgba32};

fn main() -> Result<(), std::io::Error> {
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
    p.fill(&fish, FillRule::NonZero).over(Rgba32::rgb(127, 96, 96));
    p.stroke(&fish).over(Rgba32::rgb(255, 208, 208));
    p.stroke(&eye).over(Rgba32::rgb(0, 0, 0));
    p.write_png("./fishy.png")
}
