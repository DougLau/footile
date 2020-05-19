// fishyp.rs
use footile::{FillRule, Path, Plotter};
use pix::rgb::{Rgba8p, SRgba8};
use pix::Raster;

mod png;

fn main() -> Result<(), std::io::Error> {
    let fish = Path::default()
        .relative()
        .pen_width(3.0)
        .move_to(112.0, 24.0)
        .line_to(-32.0, 24.0)
        .cubic_to(-96.0, -48.0, -96.0, 80.0, 0.0, 32.0)
        .line_to(32.0, 24.0)
        .line_to(-16.0, -40.0)
        .close()
        .finish();
    let eye = Path::default()
        .relative()
        .pen_width(2.0)
        .move_to(24.0, 48.0)
        .line_to(8.0, 8.0)
        .move_to(0.0, -8.0)
        .line_to(-8.0, 8.0)
        .finish();

    // Emulate Non-owned Pointer to Vulkan Buffer:
    let mut array = [Rgba8p::new(0, 0, 0, 0); 128 * 128];
    let buffer: *mut Rgba8p = array.as_mut_ptr();

    // Safely convert our Vulkan Pointer into a Box<[T]>, then into a Vec<T>.
    // This is safe because slice & box are fat ptrs.
    let slice: &mut [Rgba8p] =
        unsafe { std::slice::from_raw_parts_mut(buffer, 128 * 128) };
    let v: Box<[Rgba8p]> =
        unsafe { std::mem::transmute::<_, Box<[Rgba8p]>>(slice) };

    // Plot on the buffer.
    let r = Raster::<Rgba8p>::with_pixels(128, 128, v);
    let mut p = Plotter::new(r);
    p.fill(FillRule::NonZero, &fish, Rgba8p::new(127, 96, 96, 255));
    p.stroke(&fish, Rgba8p::new(255, 208, 208, 255));
    p.stroke(&eye, Rgba8p::new(0, 0, 0, 255));

    let r = p.raster();
    let out = Raster::<SRgba8>::with_raster(&r);
    png::write(&out, "./fishyp.png")?;

    // Convert raster back to slice to avoid double free.
    let b: Box<[Rgba8p]> = r.into();
    let _: &mut [Rgba8p] = unsafe { std::mem::transmute(b) };

    Ok(())
}
