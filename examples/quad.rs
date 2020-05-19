// quad.rs
use footile::{Path, Plotter};
use pix::matte::Matte8;
use pix::Raster;

mod png;

fn main() -> Result<(), std::io::Error> {
    let path = Path::default()
        .relative()
        .pen_width(2.0)
        .move_to(0.0, 16.0)
        .quad_to(100.0, 16.0, 0.0, 32.0)
        .finish();
    let r = Raster::with_clear(64, 64);
    let mut p = Plotter::new(r);
    png::write_matte(p.stroke(&path, Matte8::new(255)), "./quad.png")
}
