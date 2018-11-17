// stroke2.rs
extern crate footile;

use footile::{Gray8,PathBuilder,Plotter};

fn main() -> Result<(), std::io::Error> {
    let path = PathBuilder::new().relative().pen_width(6f32)
                           .move_to(16f32, 15f32)
                           .line_to(32f32, 1f32)
                           .line_to(-32f32, 1f32)
                           .line_to(32f32, 15f32)
                           .line_to(-32f32, 15f32)
                           .line_to(32f32, 1f32)
                           .line_to(-32f32, 1f32)
                           .build();
    let mut p = Plotter::<Gray8>::new(64, 64);
    p.stroke(&path).write_png("./stroke2.png")
}
