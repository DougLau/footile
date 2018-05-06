// stroke2.rs
extern crate footile;

use footile::{PathBuilder, Plotter};

fn main() {
    let path = PathBuilder::new().relative().pen_width(6f32)
                           .move_to(16f32, 15f32)
                           .line_to(32f32, 1f32)
                           .line_to(-32f32, 1f32)
                           .line_to(32f32, 15f32)
                           .line_to(-32f32, 15f32)
                           .line_to(32f32, 1f32)
                           .line_to(-32f32, 1f32)
                           .build();
    let mut p = Plotter::new(64, 64);
    p.add_ops(&path);
    p.stroke().mask().write_png("./stroke2.png").unwrap();
}
