// fishyp.rs
extern crate footile;

use footile::{FillRule,PathBuilder,Plotter,Raster,Rgba8};

fn main() -> Result<(), std::io::Error> {
    // Emulate Non-owned Pointer to Vulkan Buffer:
    let mut array: [Rgba8; 128*128] = [Rgba8::new(0,0,0,0); 128*128];
    let buffer: *mut Rgba8 = array.as_mut_ptr();

    // Convert our Vulkan Pointer into A Box<[T]>.  This is safe because slice & box are fat ptrs.
    let slice: &mut [Rgba8] = unsafe { std::slice::from_raw_parts_mut(buffer, 128*128) };
    let b: Box<[Rgba8]> = unsafe { std::mem::transmute(slice) };

    // Draw on the buffer.
    let fish = PathBuilder::new().relative().pen_width(3.0)
                           .move_to(112.0, 24.0)
                           .line_to(-32.0, 24.0)
                           .cubic_to(-96.0, -48.0, -96.0, 80.0, 0.0, 32.0)
                           .line_to(32.0, 24.0)
                           .line_to(-16.0, -40.0)
                           .close().build();
    let eye = PathBuilder::new().relative().pen_width(2.0)
                          .move_to(24.0, 48.0)
                          .line_to(8.0, 8.0)
                          .move_to(0.0, -8.0)
                          .line_to(-8.0, 8.0)
                          .build();
    let mut p = Plotter::new(128, 128);
    let mut r = Raster::owned(p.width(), p.height(), b);
    r.over(p.fill(&fish, FillRule::NonZero), Rgba8::rgb(127, 96, 96));
    r.over(p.stroke(&fish), Rgba8::rgb(255, 208, 208));
    r.over(p.stroke(&eye), Rgba8::rgb(0, 0, 0));
    r.write_png("./fishyp.png")?;

    // Convert raster back to slice to avoid double free.
    let b: Box<[Rgba8]> = r.into();
    let _: &mut [Rgba8] = unsafe { std::mem::transmute(b) };

    Ok(())
}
