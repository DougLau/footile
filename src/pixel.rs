// pixel.rs     Pixel format basics.
//
// Copyright (c) 2018  Douglas P Lau
//
use png::ColorType;
use mask::Mask;

/// Pixel format.
///
/// This determines color components and bit depth,
/// as well as the layout of pixels in memory.
pub trait PixFmt: Clone + Default {

    /// Get the PNG color type.
    fn color_type() -> ColorType;

    /// Blend pixels with an alpha mask.
    ///
    /// * `pix` Slice of pixels.
    /// * `mask` Alpha mask for compositing.
    /// * `src` Source color.
    fn over(pix: &mut [Self], mask: &Mask, src: Self);

    /// Divide alpha (remove premultiplied alpha)
    ///
    /// * `pix` Slice of pixels.
    fn divide_alpha(pix: &mut [Self]);

    /// Convert a pixel slice into a u8 slice.
    ///
    /// * `pix` Slice of pixels.
    fn as_u8_slice(pix: &[Self]) -> &[u8] {
        unsafe { pix.align_to::<u8>().1 }
    }

    /// Convert a pixel slice into a mutable u8 slice.
    ///
    /// * `pix` Slice of pixels.
    fn as_u8_slice_mut(pix: &mut [Self]) -> &mut [u8] {
        unsafe { pix.align_to_mut::<u8>().1 }
    }

    /// Convert a u8 slice into a pixel slice.
    ///
    /// * `pix` Slice of u8 pixel data.
    fn as_slice(pix: &[u8]) -> &[Self] {
        unsafe { pix.align_to::<Self>().1 }
    }

    /// Convert a u8 slice into a mutable pixel slice.
    ///
    /// * `pix` Slice of u8 pixel data.
    fn as_slice_mut(pix: &mut [u8]) -> &mut [Self] {
        unsafe { pix.align_to_mut::<Self>().1 }
    }
}
