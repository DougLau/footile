// drop.rs
extern crate footile;

use footile::{PathBuilder, Plotter};

fn main() {
    let path = PathBuilder::new().relative().pen_width(3f32)
                           .move_to(50f32, 34f32)
                           .cubic_to(4f32, 16f32, 16f32, 28f32, 0f32, 32f32)
                           .cubic_to(-16f32, -4f32, -4f32, -16f32, 0f32, -32f32)
                           .close()
                           .build();
    let mut p = Plotter::new(100, 100);
    p.add_path(path);
    p.stroke().mask().write_png("./drop.png").unwrap();
}
