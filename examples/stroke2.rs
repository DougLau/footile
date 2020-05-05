// stroke2.rs
use footile::{PathBuilder, Plotter};
use pix::rgb::{Rgba8p, SRgba8};
use pix::Raster;

mod png;

fn main() -> Result<(), std::io::Error> {
    let path = PathBuilder::default()
        .relative()
        .pen_width(6.0)
        .move_to(16.0, 15.0)
        .line_to(32.0, 1.0)
        .line_to(-32.0, 1.0)
        .line_to(32.0, 15.0)
        .line_to(-32.0, 15.0)
        .line_to(32.0, 1.0)
        .line_to(-32.0, 1.0)
        .build();
    let clr = Rgba8p::new(64, 128, 64, 255);
    let mut p = Plotter::new(Raster::with_color(64, 64, clr));
    p.stroke(&path, Rgba8p::new(255, 255, 0, 255));
    let r = Raster::<SRgba8>::with_raster(&p.raster());
    png::write(&r, "./stroke2.png")
}
