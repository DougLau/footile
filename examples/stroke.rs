// stroke.rs
extern crate footile;

use footile::{Gray8,PathBuilder,Plotter};

fn main() -> Result<(), std::io::Error> {
    let path = PathBuilder::new().relative().pen_width(5f32)
                           .move_to(16f32, 48f32)
                           .line_to(32f32, 0f32)
                           .line_to(-16f32, -32f32)
                           .close().build();
    let mut p = Plotter::<Gray8>::new(64, 64);
    p.stroke(&path).write_png("./stroke.png")
}
