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
    fn over_alpha(self, bot: Color, alpha: u8) -> Self {
        // NOTE: `bot + alpha * (top - bot)` is equivalent to
        //       `alpha * top + (1 - alpha) * bot`, but faster.
        let r = self.red()   as i32 - bot.red()   as i32;
        let g = self.green() as i32 - bot.green() as i32;
        let b = self.blue()  as i32 - bot.blue()  as i32;
        let a = self.alpha() as i32 - bot.alpha() as i32;
        let red   = (bot.red()   as i32 + scale_i32(r, alpha)) as u8;
        let green = (bot.green() as i32 + scale_i32(g, alpha)) as u8;
        let blue  = (bot.blue()  as i32 + scale_i32(b, alpha)) as u8;
        let alpha = (bot.alpha() as i32 + scale_i32(a, alpha)) as u8;
        Color::rgba(red, green, blue, alpha)
    }
}

/// Scale a u8 value by another (for alpha blending)
fn scale_i32(a: i32, b: u8) -> i32 {
    // cheap alternative to divide by 255
    let c = a * b as i32;
    (((c + 1) + (c >> 8)) >> 8) as i32
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

/// Composite packed u8 values using `over`.
#[cfg(any(target_arch="x86", target_arch="x86_64"))]
unsafe fn over_alpha_u8x16_x86(t: __m128i, b: __m128i, a: __m128i) -> __m128i {
    // Since alpha can range from 0 to 255 and (t - b) can range from -255 to
    // +255, we would need 17 bits to store the result of a multiplication.
    // Instead, shift alpha right by 1 bit (divide by 2).  Afterwards, we can
    // shift back by one less bit (in scale_i16_to_u8_x86).
    // For even lanes: b + alpha * (t - b)
    let t_even = _mm_unpacklo_epi8(t, _mm_setzero_si128());
    let b_even = _mm_unpacklo_epi8(b, _mm_setzero_si128());
    let a_even = _mm_unpacklo_epi8(a, _mm_setzero_si128());
    let a_even = _mm_srli_epi16(a_even, 1);
    let even = _mm_mullo_epi16(a_even, _mm_sub_epi16(t_even, b_even));
    let even = scale_i16_to_u8_x86(even);
    let even = _mm_add_epi16(b_even, even);
    // For odd lanes: b + alpha * (t - b)
    let t_odd = _mm_unpackhi_epi8(t, _mm_setzero_si128());
    let b_odd = _mm_unpackhi_epi8(b, _mm_setzero_si128());
    let a_odd = _mm_unpackhi_epi8(a, _mm_setzero_si128());
    let a_odd = _mm_srli_epi16(a_odd, 1);
    let odd = _mm_mullo_epi16(a_odd, _mm_sub_epi16(t_odd, b_odd));
    let odd = scale_i16_to_u8_x86(odd);
    let odd = _mm_add_epi16(b_odd, odd);
    _mm_packus_epi16(even, odd)
}

/// Scale i16 values (result of "u7" * "i9") into u8.
#[cfg(any(target_arch="x86", target_arch="x86_64"))]
unsafe fn scale_i16_to_u8_x86(v: __m128i) -> __m128i {
    // To scale into a u8, we would normally divide by 255.  This is equivalent
    // to: ((v + 1) + (v >> 8)) >> 8
    // For the last right shift, we use 7 instead to simulate multiplying by
    // 2.  This is necessary because alpha was shifted right by 1 bit to allow
    // fitting 17 bits of data into epi16 lanes.
    _mm_srai_epi16(_mm_add_epi16(_mm_add_epi16(v,
                                               _mm_set1_epi16(1)),
                                 _mm_srai_epi16(v, 8)),
                   7)
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

/// Swizzle alpha mask (xxxxxxxxxxxx3210 => 3333222211110000)
#[cfg(any(target_arch="x86", target_arch="x86_64"))]
unsafe fn swizzle_mask_x86(v: __m128i) -> __m128i {
    _mm_shuffle_epi8(v, _mm_set_epi8(3, 3, 3, 3,
                                     2, 2, 2, 2,
                                     1, 1, 1, 1,
                                     0, 0, 0, 0))
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
            let bot = Color::rgba(p[0], p[1], p[2], p[3]);
            let out = clr.over_alpha(bot, *m);
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
            // then shuffle: xxxxxxxxxxxx3210 => 3333222211110000
            let alpha = swizzle_mask_x86(_mm_set1_epi32(*src));
            // get RGBA values from dst
            let bot = _mm_loadu_si128(dst);
            // compose top over bot
            let out = over_alpha_u8x16_x86(clr, bot, alpha);
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
