// rgba8.rs     8-bit per channel RGBA pixel format.
//
// Copyright (c) 2018  Douglas P Lau
//
use png::ColorType;
use pixel::{PixFmt,lerp_u8};

#[cfg(all(target_arch = "x86", not(feature = "no-simd")))]
use std::arch::x86::*;
#[cfg(all(target_arch = "x86_64", not(feature = "no-simd")))]
use std::arch::x86_64::*;

/// 8-bit per channel RGBA [pixel format](trait.PixFmt.html).
///
/// This format has four 8-bit channels: red, green, blue and alpha.
#[derive(Clone,Copy,Debug,Default)]
#[repr(C)]
pub struct Rgba8 {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

impl From<Rgba8> for i32 {
    /// Get an i32 from a Rgba8 (alpha in high byte)
    fn from(c: Rgba8) -> i32 {
        let red   = (c.red()   as i32) << 0;
        let green = (c.green() as i32) << 8;
        let blue  = (c.blue()  as i32) << 16;
        let alpha = (c.alpha() as i32) << 24;
        red | green | blue | alpha
    }
}

impl Rgba8 {
    /// Build a color by specifying red, green, blue and alpha values.
    pub fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Rgba8 { red, green, blue, alpha }
    }
    /// Build an opaque color by specifying red, green and blue values.
    pub fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Rgba8::new(red, green, blue, 0xFF)
    }
    /// Divide alpha out of red, green and blue components.
    fn divide_alpha(self) -> Self {
        let alpha = self.alpha();
        let red   = unscale_u8(self.red(), alpha);
        let green = unscale_u8(self.green(), alpha);
        let blue  = unscale_u8(self.blue(), alpha);
        Rgba8::new(red, green, blue, alpha)
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
    fn over_alpha(self, bot: Rgba8, alpha: u8) -> Self {
        let red   = lerp_u8(self.red(),   bot.red(),   alpha);
        let green = lerp_u8(self.green(), bot.green(), alpha);
        let blue  = lerp_u8(self.blue(),  bot.blue(),  alpha);
        let alpha = lerp_u8(self.alpha(), bot.alpha(), alpha);
        Rgba8::new(red, green, blue, alpha)
    }
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

impl PixFmt for Rgba8 {
    /// Get the PNG color type.
    fn color_type() -> ColorType {
        ColorType::RGBA
    }
    /// Blend pixels with an alpha mask.
    ///
    /// * `pix` Slice of pixels.
    /// * `mask` Alpha mask for compositing.
    /// * `src` Source color.
    fn over(pix: &mut [Self], mask: &[u8], clr: Self) {
        debug_assert_eq!(pix.len(), mask.len());
        #[cfg(all(any(target_arch="x86", target_arch="x86_64"), not(feature = "no-simd")))] {
            if is_x86_feature_detected!("ssse3") {
                unsafe { over_x86(pix, mask, clr) }
                return;
            }
        }
        over_fallback(pix, mask, clr);
    }
    /// Divide alpha (remove premultiplied alpha)
    fn divide_alpha(pix: &mut [Self]) {
        for p in pix.iter_mut() {
            *p = p.divide_alpha();
        }
    }
}

/// Composite a color with a mask.
#[cfg(all(any(target_arch="x86", target_arch="x86_64"), not(feature = "no-simd")))]
unsafe fn over_x86(pix: &mut [Rgba8], mask: &[u8], clr: Rgba8) {
    debug_assert_eq!(pix.len(), mask.len());
    let len = pix.len();
    let clr = _mm_set1_epi32(clr.into());
    let src = mask.as_ptr();
    let dst = pix.as_mut_ptr();
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
#[cfg(all(any(target_arch="x86", target_arch="x86_64"), not(feature = "no-simd")))]
unsafe fn swizzle_mask_x86(v: __m128i) -> __m128i {
    _mm_shuffle_epi8(v, _mm_set_epi8(3, 3, 3, 3,
                                     2, 2, 2, 2,
                                     1, 1, 1, 1,
                                     0, 0, 0, 0))
}

/// Composite packed u8 values using `over`.
#[cfg(all(any(target_arch="x86", target_arch="x86_64"), not(feature = "no-simd")))]
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
#[cfg(all(any(target_arch="x86", target_arch="x86_64"), not(feature = "no-simd")))]
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
fn over_fallback(pix: &mut [Rgba8], mask: &[u8], clr: Rgba8) {
    for (bot, m) in pix.iter_mut().zip(mask) {
        let mut out = clr.over_alpha(*bot, *m);
        *bot = out;
    }
}
