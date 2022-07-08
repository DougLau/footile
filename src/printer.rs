use crate::imgbuf::{matte_src_over_even_odd, matte_src_over_non_zero};
use pix::matte::Matte8;
use pix::ops::SrcOver;
use pix::{
    chan::{Ch8, Linear, Premultiplied},
    el::Pixel,
};
use std::any::TypeId;

/// a "Printing" head for plotter
/// this struct determines how to fill in pixels
pub trait Printer<P>
where
    P: Pixel<Chan = Ch8, Alpha = Premultiplied, Gamma = Linear>,
{
    /// Accumulate scan area with non-zero fill rule.
    fn scan_non_zero<'a>(
        &mut self,
        sgn_area: &'a mut [i16],
        dst: &mut [P],
        y_row: i32,
    ) -> ();

    /// Accumulate scan area with even-odd fill rule.
    fn scan_even_odd<'a>(
        &mut self,
        sgn_area: &'a mut [i16],
        dst: &mut [P],
        y_row: i32,
    ) -> ();

    /// Cast an i16 to a u8 with saturation
    fn saturating_cast_i16_u8(v: i16) -> u8 {
        v.max(0).min(255) as u8
    }
}

pub struct ColorPrinter<P>
where
    P: Pixel<Chan = Ch8, Alpha = Premultiplied, Gamma = Linear>,
{
    pub clr: P,
}

impl<P> Printer<P> for ColorPrinter<P>
where
    P: Pixel<Chan = Ch8, Alpha = Premultiplied, Gamma = Linear>,
{
    fn scan_even_odd<'a>(
        &mut self,
        sgn_area: &'a mut [i16],
        dst: &mut [P],
        _: i32,
    ) -> () {
        let clr = self.clr;
        if TypeId::of::<P>() == TypeId::of::<Matte8>() {
            // FIXME: only if clr is Matte8::new(255)
            matte_src_over_even_odd(dst, sgn_area);
            return;
        }
        let mut sum = 0;
        for (d, s) in dst.iter_mut().zip(sgn_area.iter_mut()) {
            sum += *s;
            *s = 0;
            let v = sum & 0xFF;
            let odd = sum & 0x100;
            let c = (v - odd).abs();
            let alpha = Ch8::from(Self::saturating_cast_i16_u8(c));
            d.composite_channels_alpha(&clr, SrcOver, &alpha);
        }
    }

    fn scan_non_zero<'a>(
        &mut self,
        sgn_area: &'a mut [i16],
        dst: &mut [P],
        _: i32,
    ) -> () {
        let clr = self.clr;
        if TypeId::of::<P>() == TypeId::of::<Matte8>() {
            // FIXME: only if clr is Matte8::new(255)
            matte_src_over_non_zero(dst, sgn_area);
            return;
        }
        let mut sum = 0;
        for (d, s) in dst.iter_mut().zip(sgn_area.iter_mut()) {
            sum += *s;
            *s = 0;
            let alpha = Ch8::from(Self::saturating_cast_i16_u8(sum));
            d.composite_channels_alpha(&clr, SrcOver, &alpha);
        }
    }
}
