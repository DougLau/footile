// drop.rs
use footile::{FillRule, Path2D, Plotter};
use pix::Raster;
use pix::gray::{Graya8p, SGray8};

mod png;

fn main() -> Result<(), std::io::Error> {
    let path = Path2D::default()
        .relative()
        .pen_width(3.0)
        .move_to(50.0, 34.0)
        .cubic_to(4.0, 16.0, 16.0, 28.0, 0.0, 32.0)
        .cubic_to(-16.0, -4.0, -4.0, -16.0, 0.0, -32.0)
        .close()
        .finish();
    let r = Raster::<Graya8p>::with_clear(100, 100);
    let mut p = Plotter::new(r);
    p.fill(FillRule::NonZero, &path, Graya8p::new(128, 255));
    p.stroke(&path, Graya8p::new(255, 255));

    let r = Raster::<SGray8>::with_raster(&p.raster());
    png::write(&r, "./drop.png")
}
