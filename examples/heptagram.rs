// heptagram.rs
use footile::{FillRule, PathBuilder, Plotter, Transform};

mod png;

fn main() -> Result<(), std::io::Error> {
    const PI: f32 = std::f32::consts::PI;
    let mut p = Plotter::new(100, 100);
    let h = (p.width() / 2u32) as f32;
    let q = h / 2.0;
    p.set_transform(Transform::new_scale(h, h).translate(q, q));
    let mut pb = PathBuilder::default();
    pb = pb.move_to(0f32.cos(), 0f32.sin());
    for n in 1..7 {
        let th = PI * 4.0 * (n as f32) / 7.0;
        pb = pb.line_to(th.cos(), th.sin());
    }
    let path = pb.close().build();
    png::write_matte(p.fill(&path, FillRule::EvenOdd), "./heptagram.png")
}
