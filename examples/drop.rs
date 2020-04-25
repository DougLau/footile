// drop.rs
use footile::{FillRule, PathBuilder, Plotter};
use pix::gray::{Graya8p, SGray8};
use pix::ops::SrcOver;
use pix::Raster;

mod png;

fn main() -> Result<(), std::io::Error> {
    let path = PathBuilder::new()
        .relative()
        .pen_width(3.0)
        .move_to(50.0, 34.0)
        .cubic_to(4.0, 16.0, 16.0, 28.0, 0.0, 32.0)
        .cubic_to(-16.0, -4.0, -4.0, -16.0, 0.0, -32.0)
        .close()
        .build();
    let mut p = Plotter::new(100, 100);
    let mut r = Raster::<Graya8p>::with_clear(p.width(), p.height());
    r.composite_matte(
        (),
        p.fill(&path, FillRule::NonZero),
        (),
        Graya8p::new(128, 255),
        SrcOver,
    );
    p.clear_matte();
    r.composite_matte((), p.stroke(&path), (), Graya8p::new(255, 255), SrcOver);
    let r = Raster::<SGray8>::with_raster(&r);

    png::write(&r, "./drop.png")
}
