// rgba32.rs    32-bit RGBA pixel format.
//
// Copyright (c) 2018  Douglas P Lau
//
use mask::Mask;
use pixel;

#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

// Defining this allows easier testing of fallback configuration
const X86: bool = cfg!(any(target_arch="x86", target_arch="x86_64"));

/// 32-bit RGBA pixel format.
///
/// This format has four 8-bit channels: red, green, blue and alpha.
#[derive(Clone,Copy,Debug,Default)]
#[repr(C)]
pub struct Rgba32 {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

impl From<Rgba32> for i32 {
    /// Get an i32 from a Rgba32 (alpha in high byte)
    fn from(c: Rgba32) -> i32 {
        let red   = (c.red()   as i32) << 0;
        let green = (c.green() as i32) << 8;
        let blue  = (c.blue()  as i32) << 16;
        let alpha = (c.alpha() as i32) << 24;
        red | green | blue | alpha
    }
}



impl Rgba32 {
    /// Build a color by specifying red, green, blue and alpha values.
    pub fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Rgba32 { red, green, blue, alpha }
    }
    /// Build an opaque color by specifying red, green and blue values.
    pub fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Rgba32::new(red, green, blue, 0xFF)
    }
    /// Divide alpha out of red, green and blue components.
    fn divide_alpha(self) -> Self {
        let alpha = self.alpha();
        let red   = unscale_u8(self.red(), alpha);
        let green = unscale_u8(self.green(), alpha);
        let blue  = unscale_u8(self.blue(), alpha);
        Rgba32::new(red, green, blue, alpha)
    }
    /// Get the red component value.
    pub fn red(self) -> u8 {
        self.red
    }
    /// Get the green component value.
    pub fn green(self) -> u8 {
        self.green
    }
    /// Get the blue component value.
    pub fn blue(self) -> u8 {
        self.blue
    }
    /// Get the alpha component value.
    pub fn alpha(self) -> u8 {
        self.alpha
    }
    /// Composite the color with another, using "over".
    fn over_alpha(self, bot: Rgba32, alpha: u8) -> Self {
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
        Rgba32::new(red, green, blue, alpha)
    }
}

/// Scale an i32 value by a u8 (for alpha blending)
fn scale_i32(a: i32, b: u8) -> i32 {
    let c = a * b as i32;
    // cheap alternative to divide by 255
    (((c + 1) + (c >> 8)) >> 8) as i32
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

impl pixel::Format for Rgba32 {
    /// Divide alpha (remove premultiplied alpha)
    fn divide_alpha(pix: &mut [Self]) {
        for p in pix.iter_mut() {
            *p = p.divide_alpha();
        }
    }
    /// Composite mask over a pixel buffer.
    ///
    /// * `pix` Pixel buffer.
    /// * `mask` Mask for compositing.
    /// * `clr` Color to composite.
    fn over(pix: &mut [Self], mask: &Mask, clr: Self) {
        if X86 && is_x86_feature_detected!("ssse3") {
            unsafe { over_x86(pix, mask, clr) }
        } else {
            over_fallback(pix, mask, clr);
        }
    }
}

/// Composite a color with a mask.
#[cfg(any(target_arch="x86", target_arch="x86_64"))]
unsafe fn over_x86(pix: &mut [Rgba32], mask: &Mask, clr: Rgba32) {
    let clr = _mm_set1_epi32(clr.into());
    let src = mask.pixels();
    let dst = pix;
    let len = src.len().min(dst.len());
    let dst = dst.as_mut_ptr();
    let src = src.as_ptr();
    // 4 pixels at a time
    for i in (0..len).step_by(4) {
        let off = i as isize;
        let dst = dst.offset(off) as *mut __m128i;
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

/// Swizzle alpha mask (xxxxxxxxxxxx3210 => 3333222211110000)
#[cfg(any(target_arch="x86", target_arch="x86_64"))]
unsafe fn swizzle_mask_x86(v: __m128i) -> __m128i {
    _mm_shuffle_epi8(v, _mm_set_epi8(3, 3, 3, 3,
                                     2, 2, 2, 2,
                                     1, 1, 1, 1,
                                     0, 0, 0, 0))
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

/// Composite a color with a mask (slow fallback).
fn over_fallback(pix: &mut [Rgba32], mask: &Mask, clr: Rgba32) {
    for (bot, m) in pix.iter_mut().zip(mask.iter()) {
        let mut out = clr.over_alpha(*bot, *m);
        *bot = out;
    }
}
