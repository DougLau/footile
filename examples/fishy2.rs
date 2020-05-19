// fishy2.rs
use footile::{FillRule, Path, Plotter};
use pix::rgb::{Rgba8p, SRgba8};
use pix::Raster;

mod png;

fn main() -> Result<(), std::io::Error> {
    let fish = Path::default()
        .relative()
        .pen_width(3.0)
        .move_to(112.0, 24.0)
        .line_to(-32.0, 24.0)
        .cubic_to(-96.0, -48.0, -96.0, 80.0, 0.0, 32.0)
        .line_to(32.0, 24.0)
        .line_to(-16.0, -40.0)
        .close()
        .finish();
    let eye = Path::default()
        .relative()
        .pen_width(2.0)
        .move_to(24.0, 48.0)
        .line_to(8.0, 8.0)
        .move_to(0.0, -8.0)
        .line_to(-8.0, 8.0)
        .finish();
    let v = vec![Rgba8p::new(0, 0, 0, 0); 128 * 128];
    let r = Raster::<Rgba8p>::with_pixels(128, 128, v);
    let mut p = Plotter::new(r);
    p.fill(FillRule::NonZero, &fish, Rgba8p::new(127, 96, 96, 255));
    p.stroke(&fish, Rgba8p::new(255, 208, 208, 255));
    p.stroke(&eye, Rgba8p::new(0, 0, 0, 255));

    let r = Raster::<SRgba8>::with_raster(&p.raster());
    png::write(&r, "./fishy2.png")
}
