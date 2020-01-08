// fishy2.rs
use footile::{FillRule, PathBuilder, Plotter};
use pix::{Ch8, Format, AssocSRgba8, RasterBuilder, SepSRgba8};
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
    let v = vec![
        AssocSRgba8::with_rgba([
            Ch8::new(0),
            Ch8::new(0),
            Ch8::new(0),
            Ch8::new(0)
        ]);
        128 * 128
    ];
    let mut p = Plotter::new(128, 128);
    let mut r = RasterBuilder::<AssocSRgba8>::new().with_pixels(
        p.width(),
        p.height(),
        v,
    );
    raster_over(
        &mut r,
        p.fill(&fish, FillRule::NonZero),
        AssocSRgba8::new(127, 96, 96),
        0,
        0,
    );
    p.clear_mask();
    raster_over(
        &mut r,
        p.stroke(&fish),
        AssocSRgba8::new(255, 208, 208),
        0,
        0,
    );
    p.clear_mask();
    raster_over(&mut r, p.stroke(&eye), AssocSRgba8::new(0, 0, 0), 0, 0);

    let r = RasterBuilder::<SepSRgba8>::new().with_raster(&r);

    png::write(&r, "./fishy2.png")
}
