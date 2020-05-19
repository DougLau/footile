// heptagram.rs
use footile::{FillRule, Path2D, Plotter, Transform};
use pix::matte::Matte8;
use pix::Raster;

mod png;

const PI: f32 = std::f32::consts::PI;

fn main() -> Result<(), std::io::Error> {
    let r = Raster::with_clear(100, 100);
    let mut p = Plotter::new(r);
    let h = (p.width() / 2) as f32;
    let q = h / 2.0;
    p.set_transform(Transform::with_scale(h, h).translate(q, q));
    let mut pb = Path2D::default();
    pb = pb.move_to(0f32.cos(), 0f32.sin());
    for n in 1..7 {
        let th = PI * 4.0 * (n as f32) / 7.0;
        pb = pb.line_to(th.cos(), th.sin());
    }
    let path = pb.close().finish();
    p.fill(FillRule::EvenOdd, &path, Matte8::new(255));
    png::write_matte(&p.raster(), "./heptagram.png")
}
