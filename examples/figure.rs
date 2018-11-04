// figure.rs
extern crate footile;

use footile::{FillRule, PathBuilder, Plotter};

fn main() {
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
    let mut p = Plotter::new(64, 64);
    p.fill(&path, FillRule::NonZero);
    p.mask().write_png("./figure.png").unwrap();
}
