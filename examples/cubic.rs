// cubic.rs     Example plotting a cubic bézier spline.
extern crate footile;

use footile::{Gray8,PathBuilder,Plotter};

fn main() -> Result<(), std::io::Error> {
    let path = PathBuilder::new().relative().pen_width(2.0)
                           .move_to(8.0, 16.0)
                           .cubic_to(64.0, -16.0, 64.0, 48.0, 0.0, 32.0)
                           .build();
    let mut p = Plotter::<Gray8>::new(64, 64);
    p.stroke(&path).write_png("./cubic.png")
}
