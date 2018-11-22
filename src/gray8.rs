// gray8.rs     8-bit grayscale pixel format.
//
// Copyright (c) 2018  Douglas P Lau
//
use png::ColorType;
use pixel::{PixFmt,lerp_u8};

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
        let value = lerp_u8(self.value(), bot.value(), alpha);
        Gray8::new(value)
    }
}

impl PixFmt for Gray8 {
    /// Get the PNG color type.
    fn color_type() -> ColorType {
        ColorType::Grayscale
    }
    /// Blend pixels with an alpha mask.
    ///
    /// * `pix` Slice of pixels.
    /// * `mask` Alpha mask for compositing.
    /// * `src` Source color.
    fn over(pix: &mut [Self], mask: &[u8], clr: Self) {
        debug_assert_eq!(pix.len(), mask.len());
        over_fallback(pix, mask, clr);
    }
    /// Divide alpha (remove premultiplied alpha)
    fn divide_alpha(_pix: &mut [Self]) { }
}

/// Composite a color with a mask (slow fallback).
fn over_fallback(pix: &mut [Gray8], mask: &[u8], clr: Gray8) {
    for (bot, m) in pix.iter_mut().zip(mask) {
        *bot = clr.over_alpha(*bot, *m);
    }
}
