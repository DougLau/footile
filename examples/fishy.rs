// fishy.rs
use footile::{FillRule, PathBuilder, Plotter};
use pix::{RasterBuilder, Rgba8, AlphaMode};
use pixops::raster_over;

mod png;

fn main() -> Result<(), std::io::Error> {
    let fish = PathBuilder::new()
        .relative()
        .pen_width(3.0)
        .move_to(112.0, 24.0)
        .line_to(-32.0, 24.0)
        .cubic_to(-96.0, -48.0, -96.0, 80.0, 0.0, 32.0)
        .line_to(32.0, 24.0)
        .line_to(-16.0, -40.0)
        .close()
        .build();
    let eye = PathBuilder::new()
        .relative()
        .pen_width(2.0)
        .move_to(24.0, 48.0)
        .line_to(8.0, 8.0)
        .move_to(0.0, -8.0)
        .line_to(-8.0, 8.0)
        .build();
    let mut p = Plotter::new(128, 128);
    let mut r = RasterBuilder::<Rgba8>::new()
        .alpha_mode(AlphaMode::Associated)
        .with_clear(p.width(), p.height());
    raster_over(
        &mut r,
        p.fill(&fish, FillRule::NonZero),
        Rgba8::new(127, 96, 96),
        0,
        0,
    );
    p.clear_mask();
    raster_over(&mut r, p.stroke(&fish), Rgba8::new(255, 208, 208), 0, 0);
    p.clear_mask();
    raster_over(&mut r, p.stroke(&eye), Rgba8::new(0, 0, 0), 0, 0);

    let r = RasterBuilder::<Rgba8>::new()
        .with_raster(&r);

    png::write(&r, "./fishy.png")
}
