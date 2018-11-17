// pixel.rs     Pixel format basics.
//
// Copyright (c) 2018  Douglas P Lau
//
use std::mem;
use std::slice;
use png::ColorType;
use mask::Mask;

pub trait Format: Clone + Default {
    /// Composite a slice of pixels with a mask, using "over".
    ///
    /// * `pixels` Slice of pixels.
    /// * `mask` Mask for compositing.
    /// * `src` Source color.
    fn over(&mut [Self], &Mask, Self);

    /// Divide out precomputed alpha.
    fn divide_alpha(&mut [Self]);

    /// Get the PNG color type.
    fn color_type() -> ColorType {
        ColorType::RGBA
    }

    /// Transmute a pixel slice into a u8 slice.
    fn as_u8_slice(pix: &[Self]) -> &[u8] {
        let len = pix.len() * mem::size_of::<Self>();
        let b = (pix as *const [Self]) as *const u8;
        unsafe {
            slice::from_raw_parts(b, len)
        }
    }
}
