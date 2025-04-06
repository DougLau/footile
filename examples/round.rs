// round.rs
use footile::{JoinStyle, Path2D, Plotter};
use pix::Raster;
use pix::matte::Matte8;

mod png;

fn main() -> Result<(), std::io::Error> {
    let path = Path2D::default()
        .relative()
        .pen_width(40.0)
        .move_to(10.0, 60.0)
        .line_to(50.0, 0.0)
        .line_to(0.0, -50.0)
        .finish();
    let mut p = Plotter::new(Raster::with_clear(100, 100));
    p.set_join(JoinStyle::Round);
    png::write_matte(p.stroke(&path, Matte8::new(255)), "./round.png")
}
