// quad.rs     Example plotting a quadratic bézier spline.
extern crate footile;

use footile::{Gray8,PathBuilder,Plotter};

fn main() -> Result<(), std::io::Error> {
    let path = PathBuilder::new().relative().pen_width(2f32)
                           .move_to(0f32, 16f32)
                           .quad_to(100f32, 16f32, 0f32, 32f32)
                           .build();
    let mut p = Plotter::<Gray8>::new(64, 64);
    p.stroke(&path).write_png("./quad.png")
}
