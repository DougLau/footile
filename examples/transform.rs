use footile::{Path2D, Plotter, TransformOp};
use pix::rgb::{Rgba8p, SRgba8};
use pix::Raster;
use pointy::Transform;

mod png;

fn main() -> Result<(), std::io::Error> {
    let mut r = Raster::<Rgba8p>::with_clear(1000, 1000);
    r = render(1, r, Rgba8p::new(200, 0, 0, 255 / 2));
    r = render(2, r, Rgba8p::new(100, 150, 50, 255 / 2));
    r = render(3, r, Rgba8p::new(0, 200, 0, 255 / 2));
    r = render(4, r, Rgba8p::new(0, 0, 200, 255 / 2));
    r = render(5, r, Rgba8p::new(200, 200, 0, 255 / 2));
    r = render(6, r, Rgba8p::new(200, 0, 200, 255 / 2));
    r = render(7, r, Rgba8p::new(200, 200, 200, 255 / 2));
    let out = Raster::<SRgba8>::with_raster(&r);
    png::write(&out, "./transform.png")
}

fn render(n: usize, r: Raster<Rgba8p>, c: Rgba8p) -> Raster<Rgba8p> {
    let mut path = Path2D::default().absolute().pen_width(7.0 / n as f32);
    for i in 0..n {
        path = path
            .transform(TransformOp::None)
            .move_to(10.0, 10.0)
            .transform(TransformOp::Translate(-10.0, -10.0))
            .transform(TransformOp::Rotate(
                (-2.0 as f32).to_radians() * i as f32,
            ))
            .transform(TransformOp::Scale(
                0.1 * i as f32 + 1.0,
                0.1 * i as f32 + 1.0,
            ))
            .transform(TransformOp::Translate(10.0, 10.0))
            .cubic_to(30.0, 10.0, 50.0, 50.0, 10.0, 50.0)
    }
    let ops = path.finish();
    let mut p = Plotter::new(r);
    p.set_transform(Transform::<f32>::with_scale(3.0, 3.0));
    p.stroke(&ops, c).to_owned()
}
