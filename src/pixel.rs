// pixel.rs     Pixel format basics.
//
// Copyright (c) 2018  Douglas P Lau
//
use png::ColorType;

/// Pixel format.
///
/// This determines color components and bit depth,
/// as well as the layout of pixels in memory.
///
/// * [Gray8](struct.Gray8.html)
/// * [Rgb8](struct.Rgb8.html)
/// * [Rgba8](struct.Rgba8.html)
pub trait PixFmt: Clone + Default {

    /// Get the PNG color type.
    fn color_type() -> ColorType;

    /// Blend pixels with an alpha mask.
    ///
    /// * `pix` Slice of pixels.
    /// * `mask` Alpha mask for compositing.
    /// * `src` Source color.
    fn over(pix: &mut [Self], mask: &[u8], src: Self);

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

/// Linear interpolation of u8 values (for alpha blending)
pub fn lerp_u8(src: u8, dst: u8, alpha: u8) -> u8 {
    // NOTE: Alpha blending euqation is: `alpha * top + (1 - alpha) * bot`
    //       This is equivalent to lerp: `bot + alpha * (top - bot)`
    let src = src as i32;
    let dst = dst as i32;
    (dst + scale_i32(alpha, src - dst)) as u8
}

/// Scale an i32 value by a u8 (for alpha blending)
fn scale_i32(a: u8, v: i32) -> i32 {
    let c = v * a as i32;
    // cheap alternative to divide by 255
    (((c + 1) + (c >> 8)) >> 8) as i32
}
