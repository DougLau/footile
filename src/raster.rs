// raster.rs    A 2D raster image.
//
// Copyright (c) 2017  Douglas P Lau
//
use std::fs::File;
use std::io;
use std::ptr;
use mask::Mask;
use palette::Rgba;
use palette::Blend;
use png;
use png::HasParameters;

/// A raster image to composite plot output.
///
/// # Example
/// ```
/// use footile::{PlotterBuilder, Raster};
/// let mut p = PlotterBuilder::new().build();
/// let mut r = Raster::new(p.width(), p.height());
/// p.pen_width(5f32)
///  .move_to(16f32, 48f32)
///  .line_to(32f32, 0f32)
///  .line_to(-16f32, -32f32)
///  .close()
///  .stroke();
/// r.composite(p.mask(), [208u8, 255u8, 208u8]);
/// ```
pub struct Raster {
    pub width  : u32,
    pub height : u32,
        pixels : Vec<u8>,
}

impl Raster {
    /// Create a new raster image.
    ///
    /// * `width` Width in pixels.
    /// * `height` Height in pixels.
    pub fn new(width: u32, height: u32) -> Raster {
        let n = width as usize * height as usize * 4 as usize;
        let pixels = vec![0u8; n];
        Raster { width: width, height: height, pixels: pixels }
    }
    /// Clear all pixels.
    pub fn clear(&mut self) {
        let len = self.pixels.len();
        unsafe {
            let pix = self.pixels.as_mut_ptr().offset(0 as isize);
            ptr::write_bytes(pix, 0u8, len);
        }
    }
    /// Composite a color with a mask.
    ///
    /// * `mask` Mask for compositing.
    /// * `clr` RGB color.
    pub fn composite(&mut self, mask: &Mask, clr: [u8; 3]) {
        for (p, m) in self.pixels.chunks_mut(4).zip(mask.iter()) {
            let src = Rgba::<f32>::new_u8(clr[0], clr[1], clr[2], *m);
            let dst = Rgba::<f32>::new_u8(p[0], p[1], p[2], p[3]);
            let c = src.over(dst);
            let d = c.to_pixel::<[u8; 4]>();
            p[0] = d[0];
            p[1] = d[1];
            p[2] = d[2];
            p[3] = d[3];
        }
    }
    /// Write the raster to a PNG (portable network graphics) file.
    ///
    /// * `filename` Name of file to write.
    pub fn write_png(&self, filename: &str) -> io::Result<()> {
        let fl = File::create(filename)?;
        let ref mut bw = io::BufWriter::new(fl);
        let mut enc = png::Encoder::new(bw, self.width, self.height);
        enc.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
        let mut writer = enc.write_header()?;
        writer.write_image_data(&self.pixels[..])?;
        Ok(())
    }
}
