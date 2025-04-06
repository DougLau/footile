// curve.rs
use footile::{Path2D, Plotter};
use pix::Raster;
use pix::matte::Matte8;

mod png;

fn main() -> Result<(), std::io::Error> {
    let path = Path2D::default()
        .relative()
        .pen_width(0.0)
        .move_to(64.0, 48.0)
        .pen_width(18.0)
        .cubic_to(-64.0, -48.0, -64.0, 80.0, 0.0, 32.0)
        .finish();
    let r = Raster::with_clear(128, 128);
    let mut p = Plotter::new(r);
    png::write_matte(p.stroke(&path, Matte8::new(255)), "./curve.png")
}
