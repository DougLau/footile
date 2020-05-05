// figure.rs
use footile::{FillRule, PathBuilder, Plotter};
use pix::matte::Matte8;
use pix::Raster;

mod png;

fn main() -> Result<(), std::io::Error> {
    let path = PathBuilder::default()
        .relative()
        .move_to(4.0, 4.0)
        .line_to(28.0, 12.0)
        .line_to(28.0, -12.0)
        .line_to(-12.0, 28.0)
        .line_to(12.0, 28.0)
        .line_to(-28.0, -4.0)
        .line_to(-28.0, 4.0)
        .line_to(12.0, -28.0)
        .close()
        .build();
    let r = Raster::with_clear(64, 64);
    let mut p = Plotter::new(r);
    p.fill(FillRule::NonZero, &path, Matte8::new(255));
    png::write_matte(&p.raster(), "./figure.png")
}
