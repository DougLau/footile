// mask.rs    A 2D image mask.
//
// Copyright (c) 2017  Douglas P Lau
//
use std::fs::File;
use std::io;
use std::io::Write;
use std::ptr;

/// A Mask is an 8-bit alpha image mask.
pub struct Mask {
    width  : u32,
    height : u32,
    pixels : Vec<u8>,
}

impl Mask {
    /// Create a new mask
    pub fn new(width: u32, height: u32) -> Mask {
        let pixels = vec![0; (width * height) as usize];
        Mask { width: width, height: height, pixels: pixels }
    }
    /// Get the mask width
    pub fn width(&self) -> u32 {
        self.width
    }
    /// Get the mask height
    pub fn height(&self) -> u32 {
        self.height
    }
    /// Reset the mask (clear all pixels)
    pub fn reset(&mut self) {
        let len = self.pixels.len();
        self.fill(0, len, 0);
    }
    /// Fill a range of pixels with a single value
    pub(super) fn fill(&mut self, x: usize, len: usize, v: u8) {
        assert!(x + len <= self.pixels.len());
        unsafe {
            let pix = self.pixels.as_mut_ptr().offset(x as isize);
            ptr::write_bytes(pix, v, len);
        }
    }
    /// Set the value of one pixel
    pub(super) fn set(&mut self, x: i32, v: i32) {
        assert!(x >= 0 && (x as u32) < self.width);
        // FIXME: how to elide bounds checks
        self.pixels[x as usize] = v as u8;
    }
    /// Accumulate a scan buffer over one scan line
    pub(super) fn accumulate(&mut self, scan_buf: &Mask, row: u32) {
        assert!(scan_buf.height() == 1);
        assert!(self.width() == scan_buf.width());
        let w = self.width() as usize;
        // slicing ..w should allow bounds check elision
        let mut pix = &mut self.scan_line(row)[..w];
        let buf = &scan_buf.pixels[..w];
        for i in 0..w {
            pix[i] = pix[i].saturating_add(buf[i]);
        }
    }
    /// Get one scan line (row)
    fn scan_line(&mut self, row: u32) -> &mut [u8] {
        let s = (row * self.width) as usize;
        let t = s + self.width as usize;
        &mut self.pixels[s..t]
    }
    /// Write the mask to a PGM (portable gray map) file
    pub fn write_pgm(&self, filename: &str) -> io::Result<()> {
        let fl = File::create(filename)?;
        let mut bw = io::BufWriter::new(fl);
        let mut w = bw.get_mut();
        w.write_all(format!("P5\n{} {}\n255\n", self.width, self.height)
         .as_bytes())?;
        w.write_all(&self.pixels[..])?;
        w.flush()?;
        Ok(())
    }
}

#[test]
fn test_mask() {
    let mut m = Mask::new(10, 10);
    m.reset();
    assert!(m.width() == 10);
    assert!(m.height() == 10);
    assert!(m.pixels.len() == 100);
    m.fill(40, 20, 255u8);
    assert!(m.pixels[0] == 0u8);
    assert!(m.pixels[39] == 0u8);
    assert!(m.pixels[40] == 255u8);
    assert!(m.pixels[59] == 255u8);
    assert!(m.pixels[60] == 0u8);
    assert!(m.pixels[99] == 0u8);
}
