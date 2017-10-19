// mask.rs    A 2D image mask.
//
// Copyright (c) 2017  Douglas P Lau
//
use std::cmp;
use std::fs::File;
use std::io;
use std::io::Write;
use std::ptr;
use png;
use png::HasParameters;

/// A Mask is an image with only an 8-bit alpha channel.
///
/// It can be obtained from a [Plotter](struct.Plotter.html) after plotting.
/// A [Raster](struct.Raster.html) can be composited with a Mask.
///
/// # Example
/// ```
/// use footile::PlotterBuilder;
/// let mut p = PlotterBuilder::new().build();
/// p.move_to(10f32, 10f32)
///  .line_to(90f32, 90f32)
///  .stroke();
/// let m = p.mask();
/// ```
pub struct Mask {
    width  : u32,
    height : u32,
    pixels : Vec<u8>,
}

impl Mask {
    /// Create a new mask
    ///
    /// * `width` Width in pixels.
    /// * `height` Height in pixels.
    pub(crate) fn new(width: u32, height: u32) -> Mask {
        let pixels = vec![0; (width * height) as usize];
        Mask { width, height, pixels }
    }
    /// Get mask width.
    pub(crate) fn width(&self) -> u32 {
        self.width
    }
    /// Get mask height.
    pub(crate) fn height(&self) -> u32 {
        self.height
    }
    /// Get pixel iterator
    pub(crate) fn iter(&self) -> ::std::slice::Iter<u8> {
        self.pixels.iter()
    }
    /// Clear the mask.
    pub(crate) fn clear(&mut self) {
        let len = self.pixels.len();
        self.fill(0, len, 0);
    }
    /// Fill a range of pixels with a single value
    fn fill(&mut self, x: usize, len: usize, v: u8) {
        assert!(x + len <= self.pixels.len());
        unsafe {
            let pix = self.pixels.as_mut_ptr().offset(x as isize);
            ptr::write_bytes(pix, v, len);
        }
    }
    /// Accumulate signed area to mask.
    pub(crate) fn scan_accumulate(&mut self, sgn_area: &mut [i16], row: u32) {
        assert!(self.width == sgn_area.len() as u32);
        let w = self.width as usize;
        let scan_line = self.scan_line(row);
        let mut s = 0i16;
        for x in 0..w {
            s += sgn_area[x];
            sgn_area[x] = 0i16;
            let p = cmp::max(0, cmp::min(255, s)) as u8;
            scan_line[x] = p;
        }
    }
    /// Get one scan line (row)
    fn scan_line(&mut self, row: u32) -> &mut [u8] {
        let s = (row * self.width) as usize;
        let t = s + self.width as usize;
        &mut self.pixels[s..t]
    }
    /// Write the mask to a PGM (portable gray map) file.
    ///
    /// * `filename` Name of file to write.
    pub fn write_pgm(&self, filename: &str) -> io::Result<()> {
        let fl = File::create(filename)?;
        let mut bw = io::BufWriter::new(fl);
        let w = bw.get_mut();
        w.write_all(format!("P5\n{} {}\n255\n", self.width, self.height)
         .as_bytes())?;
        w.write_all(&self.pixels[..])?;
        w.flush()?;
        Ok(())
    }
    /// Write the mask to a PNG (portable network graphics) file.
    ///
    /// * `filename` Name of file to write.
    pub fn write_png(&self, filename: &str) -> io::Result<()> {
        let fl = File::create(filename)?;
        let ref mut bw = io::BufWriter::new(fl);
        let mut enc = png::Encoder::new(bw, self.width, self.height);
        enc.set(png::ColorType::Grayscale).set(png::BitDepth::Eight);
        let mut writer = enc.write_header()?;
        writer.write_image_data(&self.pixels[..])?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::Mask;
    #[test]
    fn test_mask() {
        let mut m = Mask::new(10, 10);
        m.clear();
        assert!(m.width == 10);
        assert!(m.height == 10);
        assert!(m.pixels.len() == 100);
        m.fill(40, 20, 255u8);
        assert!(m.pixels[0] == 0u8);
        assert!(m.pixels[39] == 0u8);
        assert!(m.pixels[40] == 255u8);
        assert!(m.pixels[59] == 255u8);
        assert!(m.pixels[60] == 0u8);
        assert!(m.pixels[99] == 0u8);
    }
}
