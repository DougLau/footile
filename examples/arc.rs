// arc.rs
use footile::{Path2D, Plotter};
use pix::matte::Matte8;
use pix::Raster;

mod png;

fn main() -> Result<(), std::io::Error> {
    let path = Path2D::default()
        .absolute()
        .pen_width(2.0)
        .move_to(22.2750, 64.0 / 2.0 - 22.275)
        .arc_sweep(32.0, 32.0, (300.0 as f32).to_radians())
        .finish();
    let r = Raster::with_clear(64, 64);
    let mut p = Plotter::new(r);
    png::write_matte(p.stroke(&path, Matte8::new(255)), "./arc.png")
}
