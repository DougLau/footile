// stroke.rs
extern crate footile;

use footile::{Gray8,PathBuilder,Plotter};

fn main() -> Result<(), std::io::Error> {
    let path = PathBuilder::new().relative().pen_width(5.0)
                           .move_to(16.0, 48.0)
                           .line_to(32.0, 0.0)
                           .line_to(-16.0, -32.0)
                           .close().build();
    let mut p = Plotter::<Gray8>::new(64, 64);
    p.stroke(&path).write_png("./stroke.png")
}
