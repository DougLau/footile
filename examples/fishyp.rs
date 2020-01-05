// fishyp.rs
use footile::{FillRule, PathBuilder, Plotter};
use pix::{Ch8, Format, PremulRgba8, RasterBuilder, Rgba8};
use pixops::raster_over;

mod png;

fn main() -> Result<(), std::io::Error> {
    // Emulate Non-owned Pointer to Vulkan Buffer:
    let mut array: [PremulRgba8; 128 * 128] = [PremulRgba8::with_rgba([
        Ch8::new(0),
        Ch8::new(0),
        Ch8::new(0),
        Ch8::new(0),
    ]); 128 * 128];
    let buffer: *mut PremulRgba8 = array.as_mut_ptr();

    // Safely convert our Vulkan Pointer into a Box<[T]>, then into a Vec<T>.
    // This is safe because slice & box are fat ptrs.
    let slice: &mut [PremulRgba8] =
        unsafe { std::slice::from_raw_parts_mut(buffer, 128 * 128) };
    let v: Box<[PremulRgba8]> =
        unsafe { std::mem::transmute::<_, Box<[PremulRgba8]>>(slice) };

    // Draw on the buffer.
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
    let mut r = RasterBuilder::<PremulRgba8>::new().with_pixels(
        p.width(),
        p.height(),
        v,
    );
    raster_over(
        &mut r,
        p.fill(&fish, FillRule::NonZero),
        PremulRgba8::new(127, 96, 96),
        0,
        0,
    );
    p.clear_mask();
    raster_over(
        &mut r,
        p.stroke(&fish),
        PremulRgba8::new(255, 208, 208),
        0,
        0,
    );
    p.clear_mask();
    raster_over(&mut r, p.stroke(&eye), PremulRgba8::new(0, 0, 0), 0, 0);

    let r = RasterBuilder::<Rgba8>::new().with_raster(&r);

    png::write(&r, "./fishyp.png")?;

    // Convert raster back to slice to avoid double free.
    let b: Box<[Rgba8]> = r.into();
    let _: &mut [Rgba8] = unsafe { std::mem::transmute(b) };

    Ok(())
}
