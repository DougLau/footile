// stroke.rs
extern crate footile;

use footile::{PathBuilder, Plotter};

fn main() {
    let path = PathBuilder::new().relative().pen_width(5f32)
                           .move_to(16f32, 48f32)
                           .line_to(32f32, 0f32)
                           .line_to(-16f32, -32f32)
                           .close().build();
    let mut p = Plotter::new(64, 64);
    p.add_path(path);
    p.stroke().mask().write_png("./stroke.png").unwrap();
}
