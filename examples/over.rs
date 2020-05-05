// over.rs
use footile::{PathBuilder, Plotter};
use pix::matte::Matte8;
use pix::Raster;

mod png;

fn main() -> Result<(), std::io::Error> {
    let path = PathBuilder::default()
        .relative()
        .pen_width(8.0)
        .move_to(32.0, 16.0)
        .line_to(16.0, 16.0)
        .line_to(-16.0, 16.0)
        .line_to(-16.0, -16.0)
        .line_to(16.0, -16.0)
        .line_to(0.0, 32.0)
        .build();
    let r = Raster::with_clear(64, 64);
    let mut p = Plotter::new(r);
    png::write_matte(p.stroke(&path, Matte8::new(255)), "./over.png")
}
