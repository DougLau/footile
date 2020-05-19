// stroke.rs
use footile::{Path2D, Plotter};
use pix::matte::Matte8;
use pix::Raster;

mod png;

fn main() -> Result<(), std::io::Error> {
    let path = Path2D::default()
        .relative()
        .pen_width(5.0)
        .move_to(16.0, 48.0)
        .line_to(32.0, 0.0)
        .line_to(-16.0, -32.0)
        .close()
        .finish();
    let mut p = Plotter::new(Raster::with_clear(64, 64));
    png::write_matte(p.stroke(&path, Matte8::new(255)), "./stroke.png")
}
