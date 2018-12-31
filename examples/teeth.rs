extern crate footile;

use footile::{PathBuilder,Plotter};

fn main() -> Result<(), std::io::Error> {
    let path = PathBuilder::new().relative()
                           .move_to(0.0, 8.0)
                           .line_to(8.0, 8.0)
                           .line_to(8.0, -8.0)
                           .line_to(8.0, 8.0)
                           .line_to(8.0, -8.0)
                           .line_to(8.0, 8.0)
                           .line_to(8.0, -8.0)
                           .line_to(8.0, 8.0)
                           .line_to(8.0, -8.0)
                           .move_to(-64.0, 32.0)
                           .line_to(8.0, 8.0)
                           .line_to(8.0, -8.0)
                           .line_to(8.0, 8.0)
                           .line_to(8.0, -8.0)
                           .line_to(8.0, 8.0)
                           .line_to(8.0, -8.0)
                           .line_to(8.0, 8.0)
                           .line_to(8.0, -8.0)
                           .build();
    let mut p = Plotter::new(64, 64);
    p.stroke(&path).write_png("./teeth.png")
}
