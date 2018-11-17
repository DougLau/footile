// round.rs
extern crate footile;

use footile::{Gray8,JoinStyle,PathBuilder,Plotter};

fn main() -> Result<(), std::io::Error> {
    let path = PathBuilder::new().relative().pen_width(40f32)
                           .move_to(10f32, 60f32)
                           .line_to(50f32, 0f32)
                           .line_to(0f32, -50f32)
                           .build();
    let mut p = Plotter::<Gray8>::new(100, 100);
    p.set_join(JoinStyle::Round);
    p.stroke(&path).write_png("./round.png")
}
