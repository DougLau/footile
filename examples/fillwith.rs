use pix::el::Pixel;
use footile::Ink;
struct DemoInk;
impl Ink<Rgba8p> for DemoInk {
    fn fill(&mut self, d: &mut Rgba8p, x: i32, y: i32, a: &pix::chan::Ch8) -> () {
        let clr = Rgba8p::new(
            x as u8,
            y as u8,
            (x-y).try_into().unwrap_or(0),
            255
        );
        d.composite_channels_alpha(&clr, SrcOver, a)
    }
}

// heptagram.rs
use footile::{FillRule, Path2D, Plotter};
use pix::ops::SrcOver;
use pix::rgb::{Rgba8p, SRgba8};
use pix::Raster;
use pointy::Transform;

mod png;

const PI: f32 = std::f32::consts::PI;

fn main() -> Result<(), std::io::Error> {
    let r = Raster::<Rgba8p>::with_clear(255, 255);
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
    p.fill_with(FillRule::EvenOdd, &path, DemoInk);
    let r = p.into_raster();
    let out = Raster::<SRgba8>::with_raster(&r);
    png::write(&out, "./fillwith.png")
}
