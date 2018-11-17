// over.rs
extern crate footile;

use footile::{Gray8,PathBuilder,Plotter};

fn main() -> Result<(), std::io::Error> {
    let path = PathBuilder::new().relative().pen_width(8f32)
                           .move_to(32f32, 16f32)
                           .line_to(16f32, 16f32)
                           .line_to(-16f32, 16f32)
                           .line_to(-16f32, -16f32)
                           .line_to(16f32, -16f32)
                           .line_to(0f32, 32f32)
                           .build();
    let mut p = Plotter::<Gray8>::new(64, 64);
    p.stroke(&path).write_png("./over.png")
}
