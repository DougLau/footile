use crate::imgbuf::{matte_src_over_even_odd, matte_src_over_non_zero};
use pix::matte::Matte8;
use pix::ops::SrcOver;
use pix::{
    chan::{Ch8, Linear, Premultiplied},
    el::Pixel,
};
use std::any::TypeId;

/// Ink trait
///
/// structs implementing the trait could be used with plotter
/// providing static or dynamic color
pub trait Ink<P>
where
    P: Pixel<Chan = Ch8, Alpha = Premultiplied, Gamma = Linear>,
{
    /// fill pixel
    ///
    /// * `d` destination pixel
    /// * `x` / `y` pixel coordinates
    /// * `a` alpha value
    fn fill(&mut self, d: &mut P, x: i32, y: i32, a: &Ch8) -> ();

    /// Accumulate scan area with non-zero fill rule.
    fn scan_non_zero(
        &mut self,
        dst: &mut [P],
        sgn_area: &mut [i16],
        y: i32,
    ) -> () {
        let mut sum = 0;
        for (x, (d, s)) in dst.iter_mut().zip(sgn_area.iter_mut()).enumerate() {
            sum += *s;
            *s = 0;
            let alpha = Ch8::from(Self::saturating_cast_i16_u8(sum));
            self.fill(d, x as i32, y, &alpha)
        }
    }

    /// Accumulate scan area with even-odd fill rule.
    fn scan_even_odd(
        &mut self,
        dst: &mut [P],
        sgn_area: &mut [i16],
        y: i32,
    ) -> () {
        let mut sum = 0;
        for (x, (d, s)) in dst.iter_mut().zip(sgn_area.iter_mut()).enumerate() {
            sum += *s;
            *s = 0;
            let v = sum & 0xFF;
            let odd = sum & 0x100;
            let c = (v - odd).abs();
            let alpha = Ch8::from(Self::saturating_cast_i16_u8(c));
            self.fill(d, x as i32, y, &alpha)
        }
    }

    /// Cast an i16 to a u8 with saturation
    fn saturating_cast_i16_u8(v: i16) -> u8 {
        v.max(0).min(255) as u8
    }
}
/// Color ink
///
/// paints pixels with given color
pub struct ColorInk<P>
where
    P: Pixel<Chan = Ch8, Alpha = Premultiplied, Gamma = Linear>,
{
    pub clr: P,
}

const WHITE_CH: Ch8 = Ch8::new(255);

impl<P> Ink<P> for ColorInk<P>
where
    P: Pixel<Chan = Ch8, Alpha = Premultiplied, Gamma = Linear>,
{
    fn fill(&mut self, d: &mut P, _: i32, _: i32, alpha: &Ch8) -> () {
        d.composite_channels_alpha(&self.clr, SrcOver, alpha)
    }
    fn scan_even_odd(
        &mut self,
        dst: &mut [P],
        sgn_area: &mut [i16],
        _: i32,
    ) -> () {
        if TypeId::of::<P>() == TypeId::of::<Matte8>()
            && self.clr.one() == WHITE_CH
        {
            matte_src_over_even_odd(dst, sgn_area)
        } else {
            let mut sum = 0;
            for (d, s) in dst.iter_mut().zip(sgn_area.iter_mut()) {
                sum += *s;
                *s = 0;
                let v = sum & 0xFF;
                let odd = sum & 0x100;
                let c = (v - odd).abs();
                let alpha = Ch8::from(Self::saturating_cast_i16_u8(c));
                self.fill(d, 0, 0, &alpha)
            }
        }
    }

    fn scan_non_zero(
        &mut self,
        dst: &mut [P],
        sgn_area: &mut [i16],
        _: i32,
    ) -> () {
        if TypeId::of::<P>() == TypeId::of::<Matte8>()
            && self.clr.one() == WHITE_CH
        {
            matte_src_over_non_zero(dst, sgn_area)
        } else {
            let mut sum = 0;
            for (d, s) in dst.iter_mut().zip(sgn_area.iter_mut()) {
                sum += *s;
                *s = 0;
                let alpha = Ch8::from(Self::saturating_cast_i16_u8(sum));
                self.fill(d, 0, 0, &alpha)
            }
        }
    }
}
