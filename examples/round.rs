// round.rs
extern crate footile;

use footile::{Gray8,JoinStyle,PathBuilder,Plotter};

fn main() -> Result<(), std::io::Error> {
    let path = PathBuilder::new().relative().pen_width(40.0)
                           .move_to(10.0, 60.0)
                           .line_to(50.0, 0.0)
                           .line_to(0.0, -50.0)
                           .build();
    let mut p = Plotter::<Gray8>::new(100, 100);
    p.set_join(JoinStyle::Round);
    p.stroke(&path).write_png("./round.png")
}
