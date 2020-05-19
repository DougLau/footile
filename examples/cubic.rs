// drop.rs
use footile::{Path, Plotter};
use pix::matte::Matte8;
use pix::Raster;

mod png;

fn main() -> Result<(), std::io::Error> {
    let path = Path::default()
        .relative()
        .pen_width(2.0)
        .move_to(8.0, 16.0)
        .cubic_to(64.0, -16.0, 64.0, 48.0, 0.0, 32.0)
        .finish();
    let r = Raster::with_clear(64, 64);
    let mut p = Plotter::new(r);
    png::write_matte(p.stroke(&path, Matte8::new(255)), "./cubic.png")
}
