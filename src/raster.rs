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

#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

// Defining this allows easier testing of fallback configuration
const X86: bool = cfg!(any(target_arch="x86", target_arch="x86_64"));

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
    fn multiply_alpha(self, alpha: u8) -> Self {
        let red   = scale_u8(self.red(), alpha);
        let green = scale_u8(self.green(), alpha);
        let blue  = scale_u8(self.blue(), alpha);
        let alpha = scale_u8(self.alpha(), alpha);
        Color::rgba(red, green, blue, alpha)
    }
    fn divide_alpha(self) -> Self {
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

/// A raster image.
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
/// r.color_over(p.mask(), Color::rgb(208, 255, 208));
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
    let c = (aa * bb + 255) >> 8; // cheaper approximation of divide by 255
    c as u8
}

/// Scale packed u8 values from `a` by `b` (for alpha blending)
#[cfg(any(target_arch="x86", target_arch="x86_64"))]
unsafe fn scale_u8x16_x86(a: __m128i, b: __m128i) -> __m128i {
    let a_even = _mm_unpacklo_epi8(a, _mm_setzero_si128());
    let b_even = _mm_unpacklo_epi8(b, _mm_setzero_si128());
    // For even lanes, (a * b + 255) >> 8  -- (less work than / 255)
    let even = _mm_mullo_epi16(a_even, b_even);
    let even = _mm_srli_epi16(_mm_add_epi16(even, _mm_set1_epi16(255)), 8);
    let a_odd = _mm_unpackhi_epi8(a, _mm_setzero_si128());
    let b_odd = _mm_unpackhi_epi8(b, _mm_setzero_si128());
    // For even lanes, (a * b + 255) >> 8  -- (less work than / 255)
    let odd = _mm_mullo_epi16(a_odd, b_odd);
    let odd = _mm_srli_epi16(_mm_add_epi16(odd, _mm_set1_epi16(255)), 8);
    _mm_packus_epi16(even, odd)
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

/// Swizzle alpha values (xxx3xxx2xxx1xxx0 => 3333222211110000)
#[cfg(any(target_arch="x86", target_arch="x86_64"))]
unsafe fn swizzle_alpha_x86(v: __m128i) -> __m128i {
        _mm_shuffle_epi8(v, _mm_set_epi8(15, 15, 15, 15,
                                         11, 11, 11, 11,
                                          7,  7,  7,  7,
                                          3,  3,  3,  3))
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
    /// Composite a color with a mask, using "over".
    ///
    /// * `mask` Mask for compositing.
    /// * `clr` Color to composite.
    pub fn color_over(&mut self, mask: &Mask, clr: Color) {
        if X86 && is_x86_feature_detected!("ssse3") {
            unsafe { self.color_over_x86(mask, clr) }
        } else {
            self.color_over_fallback(mask, clr);
        }
    }
    /// Composite a color with a mask (slow fallback).
    fn color_over_fallback(&mut self, mask: &Mask, clr: Color) {
        for (p, m) in self.pixels.chunks_mut(4).zip(mask.iter()) {
            let top = clr.multiply_alpha(*m);
            let bot = Color::rgba(p[0], p[1], p[2], p[3]);
            let out = top.over(bot);
            p[0] = out.red();
            p[1] = out.green();
            p[2] = out.blue();
            p[3] = out.alpha();
        }
    }
    /// Composite a color with a mask.
    #[cfg(any(target_arch="x86", target_arch="x86_64"))]
    unsafe fn color_over_x86(&mut self, mask: &Mask, clr: Color) {
        let clr = _mm_set1_epi32(clr.into());
        let src = mask.pixels();
        let dst = &mut self.pixels[..];
        let len = src.len().min(dst.len() / 4);
        let dst = dst.as_mut_ptr();
        let src = src.as_ptr();
        // 4 pixels at a time
        for i in (0..len).step_by(4) {
            let off = i as isize;
            let dst = dst.offset(off * 4) as *mut __m128i;
            let src = src.offset(off) as *const i32;
            // get 4 alpha values from src,
            // then shuffle: 0123012301230123 => 0000111122223333
            let ta = _mm_shuffle_epi8(_mm_set1_epi32(*src),
                                      _mm_set_epi8(3, 3, 3, 3,
                                                   2, 2, 2, 2,
                                                   1, 1, 1, 1,
                                                   0, 0, 0, 0));
            // premultiply alpha for `top` color
            let top = scale_u8x16_x86(clr, ta);
            // swizzle final alpha: xxx0xxx1xxx2xxx3 => 0000111122223333
            let alpha = swizzle_alpha_x86(top);
            // inverse alpha (255 - alpha)
            let ialpha = _mm_subs_epu8(_mm_set1_epi8(255u8 as i8), alpha);
            // get RGBA values from dst
            let bot = _mm_loadu_si128(dst);
            // FIXME: color fringing bug around here
            // compose top over bot
            let out = _mm_adds_epi8(top, scale_u8x16_x86(bot, ialpha));
            // store blended pixels
            _mm_storeu_si128(dst, out);
        }
    }
    /// Divide alpha (remove premultiplied alpha)
    fn divide_alpha(&mut self) {
        for p in self.pixels.chunks_mut(4) {
            let out = Color::rgba(p[0], p[1], p[2], p[3]).divide_alpha();
            p[0] = out.red();
            p[1] = out.green();
            p[2] = out.blue();
            p[3] = out.alpha();
        }
    }
    /// Write the raster to a PNG (portable network graphics) file.
    ///
    /// * `filename` Name of file to write.
    pub fn write_png(mut self, filename: &str) -> io::Result<()> {
        self.divide_alpha();
        let fl = File::create(filename)?;
        let ref mut bw = io::BufWriter::new(fl);
        let mut enc = png::Encoder::new(bw, self.width, self.height);
        enc.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
        let mut writer = enc.write_header()?;
        writer.write_image_data(&self.pixels[..])?;
        Ok(())
    }
}
