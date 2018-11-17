// drop.rs
extern crate footile;

use footile::{FillRule,Gray8,PathBuilder,Plotter};

fn main() -> Result<(), std::io::Error> {
    let path = PathBuilder::new().relative().pen_width(3f32)
                           .move_to(50f32, 34f32)
                           .cubic_to(4f32, 16f32, 16f32, 28f32, 0f32, 32f32)
                           .cubic_to(-16f32, -4f32, -4f32, -16f32, 0f32, -32f32)
                           .close()
                           .build();
    let mut p = Plotter::new(100, 100);
    p.fill(&path, FillRule::NonZero).over(Gray8::new(128));
    p.stroke(&path).over(Gray8::new(255)).write_png("./drop.png")
}
