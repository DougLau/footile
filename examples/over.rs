// over.rs
extern crate footile;

use footile::{PathBuilder, Plotter};

fn main() {
    let path = PathBuilder::new().relative().pen_width(8f32)
                           .move_to(32f32, 16f32)
                           .line_to(16f32, 16f32)
                           .line_to(-16f32, 16f32)
                           .line_to(-16f32, -16f32)
                           .line_to(16f32, -16f32)
                           .line_to(0f32, 32f32)
                           .build();
    let mut p = Plotter::new(64, 64);
    p.stroke(&path).mask().write_png("./over.png").unwrap();
}
