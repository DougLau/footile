use footile::{FillRule, PathBuilder, Plotter};
use pix::{Gray8, RasterBuilder};
use pixops::raster_over;

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
    let mut r = RasterBuilder::<pix::GrayAlpha8>::new()
        .with_clear(p.width(), p.height());
    raster_over(
        &mut r,
        p.fill(&path, FillRule::NonZero),
        Gray8::new(128),
        0,
        0,
    );
    raster_over(&mut r, p.stroke(&path), Gray8::new(255), 0, 0);

    png::write(&r, "./drop.png")
}
