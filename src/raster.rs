// raster.rs    A 2D raster image.
//
// Copyright (c) 2017-2018  Douglas P Lau
//
use std::cell::RefCell;
use std::fs::File;
use std::io;
use png;
use png::HasParameters;
use mask::Mask;
use pixel::PixFmt;

enum Pixels<'a, F: PixFmt + 'a> {
    Owned(Vec<F>),
    Borrowed(RefCell<&'a mut [F]>),
}

impl<'a, F: PixFmt> Pixels<'a, F> {
    // NOTE: using a where clause here is the only known
    //       way to describe the needed lifetime
    fn get_slice<'b>(&'b mut self) -> &'b mut [F] where 'a: 'b {
        match self {
            Pixels::Owned(v) => &mut v[..],
            Pixels::Borrowed(v) => {
                v.get_mut()
            },
        }
    }
}

/// A raster image.
///
/// The pixel data will be owned by the Raster if the `new` constructor is
/// used.  When `with_pixels` is used, it is borrowed (with a RefCell).
///
/// # Example
/// ```
/// use footile::{PathBuilder,Plotter,Raster,Rgba8};
/// let path = PathBuilder::new().pen_width(5.0)
///                        .move_to(16.0, 48.0)
///                        .line_to(32.0, 0.0)
///                        .line_to(-16.0, -32.0)
///                        .close().build();
/// let mut p = Plotter::new(100, 100);
/// let mut r = Raster::new(p.width(), p.height());
/// r.over(p.stroke(&path), Rgba8::rgb(208, 255, 208));
/// ```
pub struct Raster<'a, F: PixFmt + 'a> {
    width  : u32,
    height : u32,
    pixels : Pixels<'a, F>,
}

impl<'a, F: PixFmt> Raster<'a, F> {
    /// Create a new raster image.
    ///
    /// * `F` pixel format: [Gray8](struct.Gray8.html)
    ///                  or [Rgba8](struct.Rgba8.html).
    /// * `width` Width in pixels.
    /// * `height` Height in pixels.
    pub fn new(width: u32, height: u32) -> Raster<'a, F> {
        let n = width as usize * height as usize;
        let pixels = Pixels::Owned(vec![F::default(); n]);
        Raster { width, height, pixels }
    }
    /// Create a raster image from existing pixels.
    ///
    /// * `F` pixel format: [Gray8](struct.Gray8.html)
    ///                  or [Rgba8](struct.Rgba8.html).
    /// * `width` Width in pixels.
    /// * `height` Height in pixels.
    /// * `pixels` Pixel data.
    pub fn with_pixels<P: 'a>(width: u32, height: u32,
        pixels: RefCell<&'a mut [F]>) -> Raster<'a, F>
    {
        let n = width as usize * height as usize;
        assert_eq!(n, pixels.borrow().len());
        let pixels = Pixels::Borrowed(pixels);
        Raster { width, height, pixels }
    }
    /// Get raster width.
    pub fn width(&self) -> u32 {
        self.width
    }
    /// Get raster height.
    pub fn height(&self) -> u32 {
        self.height
    }
    /// Get a slice of all pixels.
    pub fn pixels(&'a mut self) -> &'a mut [F] {
        self.pixels.get_slice()
    }
    /// Clear all pixels.
    pub fn clear(&mut self) {
        for p in self.pixels.get_slice().iter_mut() {
            *p = F::default();
        }
    }
    /// Composite a color with a mask, using "over".
    ///
    /// * `mask` Mask for compositing.  The mask is cleared before returning.
    /// * `clr` Color to composite.
    pub fn over(&mut self, mask: &mut Mask, clr: F) {
        F::over(self.pixels.get_slice(), mask, clr);
        mask.clear();
    }
    /// Write the raster to a PNG (portable network graphics) file.
    ///
    /// * `filename` Name of file to write.
    pub fn write_png(&mut self, filename: &str) -> io::Result<()> {
        let pix = &mut self.pixels;
        let p = pix.get_slice();
        F::divide_alpha(p);
        let fl = File::create(filename)?;
        let ref mut bw = io::BufWriter::new(fl);
        let mut enc = png::Encoder::new(bw, self.width, self.height);
        enc.set(F::color_type()).set(png::BitDepth::Eight);
        let mut writer = enc.write_header()?;
        let pix = F::as_u8_slice(p);
        writer.write_image_data(pix)?;
        Ok(())
    }
}
