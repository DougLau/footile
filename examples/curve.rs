// curve.rs
extern crate footile;

use footile::{Gray8,PathBuilder,Plotter};

fn main() -> Result<(), std::io::Error> {
    let path = PathBuilder::new().relative().pen_width(0.0)
                           .move_to(64.0, 48.0)
                           .pen_width(18.0)
                           .cubic_to(-64.0, -48.0, -64.0, 80.0, 0.0, 32.0)
                           .build();
    let mut p = Plotter::<Gray8>::new(128, 128);
    p.stroke(&path).write_png("./curve.png")
}
