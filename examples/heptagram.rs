// heptagram.rs
extern crate footile;

use footile::{FillRule, PathBuilder, Plotter, Transform};

fn main() {
    const PI: f32 = std::f32::consts::PI;
    let mut p = Plotter::new(100, 100);
    let h = (p.width() / 2u32) as f32;
    let q = h / 2f32;
    p.set_transform(Transform::new_scale(h, h).translate(q, q));
    let mut pb = PathBuilder::new();
    pb = pb.move_to(0f32.cos(), 0f32.sin());
    for n in 1..7 {
        let th = PI * 4f32 * (n as f32) / 7f32;
        pb = pb.line_to(th.cos(), th.sin());
    }
    let path = pb.close().build();
    p.fill(&path, FillRule::EvenOdd);
    p.mask().write_png("./heptagram.png").unwrap();
}
