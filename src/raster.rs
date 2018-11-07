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

/// Simple RGB color
#[derive(Clone,Copy,Debug)]
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

impl From<i32> for Color {
    fn from(rgba: i32) -> Self {
        let red   = (rgba >>  0) as u8;
        let green = (rgba >>  8) as u8;
        let blue  = (rgba >> 16) as u8;
        let alpha = (rgba >> 24) as u8;
        Color::rgba(red, green, blue, alpha)
    }
}

impl From<Color> for i32 {
    fn from(c: Color) -> i32 {
        let red   = (c.red()   as i32) << 0;
        let green = (c.green() as i32) << 8;
        let blue  = (c.blue()  as i32) << 16;
        let alpha = (c.alpha() as i32) << 24;
        red | green | blue | alpha
    }
}

impl Color {
    pub fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Color { red, green, blue, alpha }
    }
    pub fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Color::rgba(red, green, blue, 0xFF)
    }
    fn premultiply_alpha(self, alpha: u8) -> Self {
        let red   = scale_u8(self.red(), alpha);
        let green = scale_u8(self.green(), alpha);
        let blue  = scale_u8(self.blue(), alpha);
        let alpha = scale_u8(self.alpha(), alpha);
        Color::rgba(red, green, blue, alpha)
    }
    fn unpremultiply_alpha(self) -> Self {
        let alpha = self.alpha();
        let red   = unscale_u8(self.red(), alpha);
        let green = unscale_u8(self.green(), alpha);
        let blue  = unscale_u8(self.blue(), alpha);
        Color::rgba(red, green, blue, alpha)
    }
    pub fn red(self) -> u8 {
        self.red
    }
    pub fn green(self) -> u8 {
        self.green
    }
    pub fn blue(self) -> u8 {
        self.blue
    }
    pub fn alpha(self) -> u8 {
        self.alpha
    }
    fn over(self, bot: Color) -> Self {
        let ia = 255 - self.alpha();
        let red   = self.red()   + scale_u8(bot.red(), ia);
        let green = self.green() + scale_u8(bot.green(), ia);
        let blue  = self.blue()  + scale_u8(bot.blue(), ia);
        let alpha = self.alpha() + scale_u8(bot.alpha(), ia);
        Color::rgba(red, green, blue, alpha)
    }
}

/// A raster image to composite plot output.
///
/// # Example
/// ```
/// use footile::{Color,PathBuilder,Plotter,Raster};
/// let path = PathBuilder::new().pen_width(5f32)
///                        .move_to(16f32, 48f32)
///                        .line_to(32f32, 0f32)
///                        .line_to(-16f32, -32f32)
///                        .close().build();
/// let mut p = Plotter::new(100, 100);
/// let mut r = Raster::new(p.width(), p.height());
/// p.stroke(&path);
/// r.composite(p.mask(), Color::rgb(208, 255, 208));
/// ```
pub struct Raster {
    width  : u32,
    height : u32,
    pixels : Vec<u8>,
}

/// Scale a u8 value by another (for alpha blending)
fn scale_u8(a: u8, b: u8) -> u8 {
    let aa = a as u32;
    let bb = b as u32;
    let c = (aa * bb + 255) >> 8; // cheaper version of divide by 255
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
        Raster { width, height, pixels }
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
    /// * `clr` Color to composite.
    pub fn composite(&mut self, mask: &Mask, clr: Color) {
        self.composite_fallback(mask, clr);
    }
    /// Composite a color with a mask (slow fallback).
    fn composite_fallback(&mut self, mask: &Mask, clr: Color) {
        for (p, m) in self.pixels.chunks_mut(4).zip(mask.iter()) {
            let top = clr.premultiply_alpha(*m);
            let bot = Color::rgb(p[0], p[1], p[2]).premultiply_alpha(p[3]);
            let out = top.over(bot).unpremultiply_alpha();
            p[0] = out.red();
            p[1] = out.green();
            p[2] = out.blue();
            p[3] = out.alpha();
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
