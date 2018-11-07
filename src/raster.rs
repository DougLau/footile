// raster.rs    A 2D raster image.
//
// Copyright (c) 2017-2018  Douglas P Lau
//
use std::fs::File;
use std::io;
use std::ptr;
use mask::Mask;
use png;
use png::HasParameters;

/// A raster image to composite plot output.
///
/// # Example
/// ```
/// use footile::{PathBuilder, Plotter, Raster};
/// let path = PathBuilder::new().pen_width(5f32)
///                        .move_to(16f32, 48f32)
///                        .line_to(32f32, 0f32)
///                        .line_to(-16f32, -32f32)
///                        .close().build();
/// let mut p = Plotter::new(100, 100);
/// let mut r = Raster::new(p.width(), p.height());
/// p.stroke(&path);
/// r.composite(p.mask(), [208u8, 255u8, 208u8]);
/// ```
pub struct Raster {
    width  : u32,
    height : u32,
    pixels : Vec<u8>,
}

/// Scale a u8 value by another (mapping range to 0-1)
fn scale_u8(a: u8, b: u8) -> u8 {
    let aa = a as u32;
    let bb = b as u32;
    let c = (aa * bb + 255) >> 8;
    c as u8
}

/// Unscale a u8
fn unscale_u8(a: u8, b: u8) -> u8 {
    if b > 0 {
        let aa = (a as u32) << 8;
        let bb = b as u32;
        (aa / bb).min(255) as u8
    } else {
        0
    }
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
        self.composite_fallback(mask, clr);
    }
    /// Composite a color with a mask (slow fallback).
    fn composite_fallback(&mut self, mask: &Mask, clr: [u8; 3]) {
        for (p, m) in self.pixels.chunks_mut(4).zip(mask.iter()) {
            let a = *m;         // src alpha
            let ia = 255 - a;   // 1 - src alpha
            let src = (scale_u8(clr[0], a),     // src red
                       scale_u8(clr[1], a),     // src green
                       scale_u8(clr[2], a),     // src blue
                       a);                      // src alpha
            let dst = (scale_u8(p[0], p[3]),    // dst red
                       scale_u8(p[1], p[3]),    // dst green
                       scale_u8(p[2], p[3]),    // dst blue
                       p[3]);                   // dst alpha
            let out = (src.0 + scale_u8(dst.0, ia),
                       src.1 + scale_u8(dst.1, ia),
                       src.2 + scale_u8(dst.2, ia),
                       src.3 + scale_u8(dst.3, ia));
            p[0] = unscale_u8(out.0, out.3);
            p[1] = unscale_u8(out.1, out.3);
            p[2] = unscale_u8(out.2, out.3);
            p[3] = out.3;
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
