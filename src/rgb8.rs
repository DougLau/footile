// rgb8.rs      8-bit per channel RGB pixel format.
//
// Copyright (c) 2018  Douglas P Lau
//
use png::ColorType;
use pixel::{PixFmt,lerp_u8};

/// 8-bit per channel RGB [pixel format](trait.PixFmt.html).
///
/// This format has three 8-bit channels: red, green and blue.
#[derive(Clone,Copy,Debug,Default)]
#[repr(C)]
pub struct Rgb8 {
    red: u8,
    green: u8,
    blue: u8,
}

impl From<Rgb8> for i32 {
    /// Get an i32 from a Rgb8
    fn from(c: Rgb8) -> i32 {
        let red   = (c.red()   as i32) << 0;
        let green = (c.green() as i32) << 8;
        let blue  = (c.blue()  as i32) << 16;
        red | green | blue
    }
}

impl Rgb8 {
    /// Build a color by specifying red, green and blue values.
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Rgb8 { red, green, blue }
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
    /// Composite the color with another, using "over".
    fn over_alpha(self, bot: Rgb8, alpha: u8) -> Self {
        let red   = lerp_u8(self.red(),   bot.red(),   alpha);
        let green = lerp_u8(self.green(), bot.green(), alpha);
        let blue  = lerp_u8(self.blue(),  bot.blue(),  alpha);
        Rgb8::new(red, green, blue)
    }
}

impl PixFmt for Rgb8 {
    /// Get the PNG color type.
    fn color_type() -> ColorType {
        ColorType::RGB
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
fn over_fallback(pix: &mut [Rgb8], mask: &[u8], clr: Rgb8) {
    for (bot, m) in pix.iter_mut().zip(mask) {
        let mut out = clr.over_alpha(*bot, *m);
        *bot = out;
    }
}
