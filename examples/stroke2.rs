// stroke2.rs
extern crate footile;

use footile::{PathBuilder,Plotter};

fn main() -> Result<(), std::io::Error> {
    let path = PathBuilder::new().relative().pen_width(6.0)
                           .move_to(16.0, 15.0)
                           .line_to(32.0, 1.0)
                           .line_to(-32.0, 1.0)
                           .line_to(32.0, 15.0)
                           .line_to(-32.0, 15.0)
                           .line_to(32.0, 1.0)
                           .line_to(-32.0, 1.0)
                           .build();
    let mut p = Plotter::new(64, 64);
    p.stroke(&path).write_png("./stroke2.png")
}
