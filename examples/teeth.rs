use footile::{Path2D, Plotter};
use pix::Raster;
use pix::matte::Matte8;

mod png;

fn main() -> Result<(), std::io::Error> {
    let path = Path2D::default()
        .relative()
        .move_to(0.0, 8.0)
        .line_to(8.0, 8.0)
        .line_to(8.0, -8.0)
        .line_to(8.0, 8.0)
        .line_to(8.0, -8.0)
        .line_to(8.0, 8.0)
        .line_to(8.0, -8.0)
        .line_to(8.0, 8.0)
        .line_to(8.0, -8.0)
        .move_to(-64.0, 32.0)
        .line_to(8.0, 8.0)
        .line_to(8.0, -8.0)
        .line_to(8.0, 8.0)
        .line_to(8.0, -8.0)
        .line_to(8.0, 8.0)
        .line_to(8.0, -8.0)
        .line_to(8.0, 8.0)
        .line_to(8.0, -8.0)
        .finish();
    let mut p = Plotter::new(Raster::with_clear(64, 64));
    png::write_matte(p.stroke(&path, Matte8::new(255)), "./teeth.png")
}
