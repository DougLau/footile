// figure.rs
extern crate footile;

use footile::{Gray8,FillRule,PathBuilder,Plotter};

fn main() -> Result<(), std::io::Error> {
    let path = PathBuilder::new().relative()
                           .move_to(4f32, 4f32)
                           .line_to(28f32, 12f32)
                           .line_to(28f32, -12f32)
                           .line_to(-12f32, 28f32)
                           .line_to(12f32, 28f32)
                           .line_to(-28f32, -4f32)
                           .line_to(-28f32, 4f32)
                           .line_to(12f32, -28f32)
                           .close().build();
    let mut p = Plotter::<Gray8>::new(64, 64);
    p.fill(&path, FillRule::NonZero);
    p.write_png("./figure.png")
}
