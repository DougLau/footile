// raster.rs    A 2D raster image.
//
// Copyright (c) 2017-2018  Douglas P Lau
//
use std::fs::File;
use std::io;
use png;
use png::HasParameters;
use mask::Mask;
use pixel::Format;

/// A raster image.
///
/// # Example
/// ```
/// use footile::{PathBuilder,Plotter,Raster,Rgba8};
/// let path = PathBuilder::new().pen_width(5f32)
///                        .move_to(16f32, 48f32)
///                        .line_to(32f32, 0f32)
///                        .line_to(-16f32, -32f32)
///                        .close().build();
/// let mut p = Plotter::<Rgba8>::new(100, 100);
/// let mut r = Raster::new(p.width(), p.height());
/// p.stroke(&path);
/// r.over(p.mask(), Rgba8::rgb(208, 255, 208));
/// ```
pub struct Raster<F: Format> {
    width  : u32,
    height : u32,
    pixels : Vec<F>,
}

/// A Raster that doesn't own pixel data.
pub struct LinkRaster<'a, F: Format + 'a> {
    width  : u32,
    height : u32,
    pixels : &'a mut [F],
}

impl<'a, F: Format + 'a> LinkRaster<'a, F> {
    /// Create a new linked raster image.
    ///
    /// * `F` pixel format: [Gray8](struct.Gray8.html)
    ///                  or [Rgba8](struct.Rgba8.html).
    /// * `width` Width in pixels.
    /// * `height` Height in pixels.
    /// * `pixels` The pixel data.
    pub fn new(width: u32, height: u32, pixels: &'a mut [F]) -> LinkRaster<F> {
        let n = width as usize * height as usize;
        assert_eq!(n, pixels.len());
        LinkRaster { width, height, pixels }
    }
    /// Create a linked raster from a raster.
    pub fn from(r: &mut Raster<F>) -> LinkRaster<F> {
        let width = r.width;
        let height = r.height;
        let pixels = &mut r.pixels;
        LinkRaster { width, height, pixels }
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
    pub fn pixels(&self) -> &[F] {
        self.pixels
    }
    /// Clear all pixels.
    pub fn clear(&mut self) {
        for p in self.pixels.iter_mut() {
            *p = F::default();
        }
    }
    /// Composite a color with a mask, using "over".
    ///
    /// * `mask` Mask for compositing.
    /// * `clr` Color to composite.
    pub fn over(&mut self, mask: &Mask, clr: F) {
        F::over(self.pixels, mask, clr);
    }
}

impl<F: Format> Raster<F> {
    /// Create a new raster image.
    ///
    /// * `F` pixel format: [Gray8](struct.Gray8.html)
    ///                  or [Rgba8](struct.Rgba8.html).
    /// * `width` Width in pixels.
    /// * `height` Height in pixels.
    pub fn new(width: u32, height: u32) -> Raster<F> {
        let n = width as usize * height as usize;
        let pixels = vec![F::default(); n];
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
    pub fn pixels(&self) -> &[F] {
        &self.pixels
    }
    /// Clear all pixels.
    pub fn clear(&mut self) {
        LinkRaster::from(self).clear();
    }
    /// Composite a color with a mask, using "over".
    ///
    /// * `mask` Mask for compositing.
    /// * `clr` Color to composite.
    pub fn over(&mut self, mask: &Mask, clr: F) {
        LinkRaster::from(self).over(mask, clr);
    }
    /// Write the raster to a PNG (portable network graphics) file.
    ///
    /// * `filename` Name of file to write.
    pub fn write_png(mut self, filename: &str) -> io::Result<()> {
        F::divide_alpha(&mut self.pixels);
        let fl = File::create(filename)?;
        let ref mut bw = io::BufWriter::new(fl);
        let mut enc = png::Encoder::new(bw, self.width, self.height);
        enc.set(F::color_type()).set(png::BitDepth::Eight);
        let mut writer = enc.write_header()?;
        let pix = F::as_u8_slice(&self.pixels);
        writer.write_image_data(pix)?;
        Ok(())
    }
}
