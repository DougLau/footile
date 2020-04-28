// fishy2.rs
use footile::{FillRule, PathBuilder, Plotter};
use pix::ops::SrcOver;
use pix::rgb::{Rgba8p, SRgba8};
use pix::Raster;

mod png;

fn main() -> Result<(), std::io::Error> {
    let fish = PathBuilder::default()
        .relative()
        .pen_width(3.0)
        .move_to(112.0, 24.0)
        .line_to(-32.0, 24.0)
        .cubic_to(-96.0, -48.0, -96.0, 80.0, 0.0, 32.0)
        .line_to(32.0, 24.0)
        .line_to(-16.0, -40.0)
        .close()
        .build();
    let eye = PathBuilder::default()
        .relative()
        .pen_width(2.0)
        .move_to(24.0, 48.0)
        .line_to(8.0, 8.0)
        .move_to(0.0, -8.0)
        .line_to(-8.0, 8.0)
        .build();
    let v = vec![Rgba8p::new(0, 0, 0, 0); 128 * 128];
    let mut p = Plotter::new(128, 128);
    let mut r = Raster::<Rgba8p>::with_pixels(p.width(), p.height(), v);
    let clr = Rgba8p::new(127, 96, 96, 255);
    r.composite_matte((), p.fill(&fish, FillRule::NonZero), (), clr, SrcOver);
    p.clear_matte();
    let clr = Rgba8p::new(255, 208, 208, 255);
    r.composite_matte((), p.stroke(&fish), (), clr, SrcOver);
    p.clear_matte();
    let clr = Rgba8p::new(0, 0, 0, 255);
    r.composite_matte((), p.stroke(&eye), (), clr, SrcOver);

    let r = Raster::<SRgba8>::with_raster(&r);
    png::write(&r, "./fishy2.png")
}
