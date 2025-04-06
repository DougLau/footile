// stroke2.rs
use footile::{Path2D, Plotter};
use pix::Raster;
use pix::rgb::{Rgba8p, SRgba8};

mod png;

fn main() -> Result<(), std::io::Error> {
    let path = Path2D::default()
        .relative()
        .pen_width(6.0)
        .move_to(16.0, 15.0)
        .line_to(32.0, 1.0)
        .line_to(-32.0, 1.0)
        .line_to(32.0, 15.0)
        .line_to(-32.0, 15.0)
        .line_to(32.0, 1.0)
        .line_to(-32.0, 1.0)
        .finish();
    let clr = Rgba8p::new(64, 128, 64, 255);
    let mut p = Plotter::new(Raster::with_color(64, 64, clr));
    p.stroke(&path, Rgba8p::new(255, 255, 0, 255));
    let r = Raster::<SRgba8>::with_raster(&p.raster());
    png::write(&r, "./stroke2.png")
}
