// gray8.rs     8-bit grayscale pixel format.
//
// Copyright (c) 2018  Douglas P Lau
//
use png::ColorType;
use mask::Mask;
use pixel;

/// 8-bit grayscale [pixel format](trait.PixFmt.html).
///
/// This pixel format is for 8-bit grayscale with no alpha channel.
#[derive(Clone,Copy,Debug,Default)]
#[repr(C)]
pub struct Gray8 {
    value: u8,
}

impl Gray8 {
    /// Build a gray8 value.
    pub fn new(value: u8) -> Self {
        Gray8 { value }
    }
    /// Get the component value.
    pub fn value(self) -> u8 {
        self.value
    }
    /// Composite the color with another, using "over".
    fn over_alpha(self, bot: Gray8, alpha: u8) -> Self {
        // NOTE: `bot + alpha * (top - bot)` is equivalent to
        //       `alpha * top + (1 - alpha) * bot`, but faster.
        let v = self.value() as i32 - bot.value() as i32;
        let value = (bot.value() as i32 + scale_i32(v, alpha)) as u8;
        Gray8::new(value)
    }
}

/// Scale an i32 value by a u8 (for alpha blending)
fn scale_i32(a: i32, b: u8) -> i32 {
    let c = a * b as i32;
    // cheap alternative to divide by 255
    (((c + 1) + (c >> 8)) >> 8) as i32
}

impl pixel::PixFmt for Gray8 {
    /// Get the PNG color type.
    fn color_type() -> ColorType {
        ColorType::Grayscale
    }
    /// Blend pixels with an alpha mask.
    ///
    /// * `pix` Slice of pixels.
    /// * `mask` Alpha mask for compositing.
    /// * `src` Source color.
    fn over(pix: &mut [Self], mask: &Mask, clr: Self) {
        over_fallback(pix, mask, clr);
    }
    /// Divide alpha (remove premultiplied alpha)
    fn divide_alpha(_pix: &mut [Self]) { }
}

/// Composite a color with a mask (slow fallback).
fn over_fallback(pix: &mut [Gray8], mask: &Mask, clr: Gray8) {
    for (bot, m) in pix.iter_mut().zip(mask.pixels()) {
        *bot = clr.over_alpha(*bot, *m);
    }
}
