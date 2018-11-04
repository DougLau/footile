// round.rs
extern crate footile;

use footile::{JoinStyle, PathBuilder, Plotter};

fn main() {
    let path = PathBuilder::new().relative().pen_width(40f32)
                           .move_to(10f32, 60f32)
                           .line_to(50f32, 0f32)
                           .line_to(0f32, -50f32)
                           .build();
    let mut p = Plotter::new(100, 100);
    p.set_join(JoinStyle::Round);
    p.stroke(&path).mask().write_png("./round.png").unwrap();
}
