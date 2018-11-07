// imgbuf.rs        Functions for blending image buffers.
//
// Copyright (c) 2017-2018  Douglas P Lau
//

#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

// Defining this allows easier testing of fallback configuration
const X86: bool = cfg!(any(target_arch="x86", target_arch="x86_64"));

/// Accumulate signed area with non-zero fill rule.
/// Source buffer is zeroed upon return.
///
/// * `dst` Destination buffer.
/// * `src` Source buffer.
pub(crate) fn accumulate_non_zero(dst: &mut [u8], src: &mut [i16]) {
    assert!(dst.len() <= src.len());
    if X86 && is_x86_feature_detected!("ssse3") {
        unsafe { accumulate_non_zero_x86(dst, src) }
    } else {
        accumulate_non_zero_fallback(dst, src)
    }
}

/// Accumulate signed area with non-zero fill rule.
fn accumulate_non_zero_fallback(dst: &mut [u8], src: &mut [i16]) {
    let mut sum = 0;
    for (d, s) in dst.iter_mut().zip(src.iter_mut()) {
        sum += *s;
        *s = 0;
        *d = saturating_cast_i16_u8(sum);
    }
}

/// Cast an i16 to a u8 with saturation
fn saturating_cast_i16_u8(v: i16) -> u8 {
    v.max(0).min(255) as u8
}

/// Accumulate signed area with non-zero fill rule.
#[cfg(any(target_arch="x86", target_arch="x86_64"))]
unsafe fn accumulate_non_zero_x86(dst: &mut [u8], src: &mut [i16]) {
    let zero = _mm_setzero_si128();
    let mut sum = zero;
    let len = dst.len().min(src.len());
    let dst = dst.as_mut_ptr();
    let src = src.as_mut_ptr();
    for i in (0..len).step_by(8) {
        let off = i as isize;
        let d = dst.offset(off) as *mut __m128i;
        let s = src.offset(off) as *mut __m128i;
        // get 8 values from src
        let mut a = _mm_loadu_si128(s);
        // zeroing now is faster than memset later
        _mm_storeu_si128(s, zero);
        //   a7 a6 a5 a4 a3 a2 a1 a0
        // + a3 a2 a1 a0 __ __ __ __
        a = _mm_add_epi16(a, _mm_slli_si128(a, 8));
        // + a5 a4 a3 a2 a1 a0 __ __
        // + a1 a0 __ __ __ __ __ __
        a = _mm_add_epi16(a, _mm_slli_si128(a, 4));
        // + a6 a5 a4 a3 a2 a1 a0 __
        // + a2 a1 a0 __ __ __ __ __
        // + a4 a3 a2 a1 a0 __ __ __
        // + a0 __ __ __ __ __ __ __
        a = _mm_add_epi16(a, _mm_slli_si128(a, 2));
        // add in previous sum
        a = _mm_add_epi16(a, sum);
        // pack to u8 using saturation
        let b = _mm_packus_epi16(a, a);
        _mm_storel_epi64(d, b);
        // shuffle sum into all lanes
        sum = _mm_shuffle_epi8(a, _mm_set1_epi16(0x0F_0E));
    }
}

/// Accumulate signed area with even-odd fill rule.
/// Source buffer is zeroed upon return.
///
/// * `dst` Destination buffer.
/// * `src` Source buffer.
pub(crate) fn accumulate_odd(dst: &mut [u8], src: &mut [i16]) {
    assert!(dst.len() <= src.len());
    if X86 && is_x86_feature_detected!("ssse3") {
        unsafe { accumulate_odd_x86(dst, src) }
    } else {
        accumulate_odd_fallback(dst, src)
    }
}

/// Accumulate signed area with even-odd fill rule.
fn accumulate_odd_fallback(dst: &mut [u8], src: &mut [i16]) {
    let mut sum = 0;
    for (d, s) in dst.iter_mut().zip(src.iter_mut()) {
        sum += *s;
        *s = 0;
        let v = sum & 0xFF;
        let odd = sum & 0x100;
        let c = (v - odd).abs();
        *d = saturating_cast_i16_u8(c);
    }
}

/// Accumulate signed area with even-odd fill rule.
#[cfg(any(target_arch="x86", target_arch="x86_64"))]
unsafe fn accumulate_odd_x86(dst: &mut [u8], src: &mut [i16]) {
    let zero = _mm_setzero_si128();
    let mut sum = zero;
    for (d, s) in dst.chunks_mut(8).zip(src.chunks_mut(8)) {
        let d = d.as_mut_ptr() as *mut __m128i;
        let s = s.as_mut_ptr() as *mut __m128i;
        // get 8 values from src
        let mut a = _mm_loadu_si128(s);
        // zeroing now is faster than memset later
        _mm_storeu_si128(s, zero);
        //   a7 a6 a5 a4 a3 a2 a1 a0
        // + a3 a2 a1 a0 __ __ __ __
        a = _mm_add_epi16(a, _mm_slli_si128(a, 8));
        // + a5 a4 a3 a2 a1 a0 __ __
        // + a1 a0 __ __ __ __ __ __
        a = _mm_add_epi16(a, _mm_slli_si128(a, 4));
        // + a6 a5 a4 a3 a2 a1 a0 __
        // + a2 a1 a0 __ __ __ __ __
        // + a4 a3 a2 a1 a0 __ __ __
        // + a0 __ __ __ __ __ __ __
        a = _mm_add_epi16(a, _mm_slli_si128(a, 2));
        // add in previous sum
        a = _mm_add_epi16(a, sum);
        let mut v = _mm_and_si128(a, _mm_set1_epi16(0xFF));
        let odd = _mm_and_si128(a, _mm_set1_epi16(0x100));
        v = _mm_sub_epi16(v, odd);
        v = _mm_abs_epi16(v);
        // pack to u8 using saturation
        let b = _mm_packus_epi16(v, v);
        _mm_storel_epi64(d, b);
        // shuffle sum into all lanes
        sum = _mm_shuffle_epi8(a, _mm_set1_epi16(0x0F_0E));
    }
}

#[cfg(test)]
mod test {
    use super::{accumulate_non_zero, accumulate_odd};
    #[test]
    fn non_zero() {
        let mut a = [0u8; 3000];
        let mut b = [0i16; 3000];
        b[0] = 200i16;
        accumulate_non_zero(&mut a, &mut b);
        for ai in a.iter() {
            assert!(*ai == 200);
        }
        let mut c = [0u8; 5000];
        let mut d = [0i16; 5000];
        d[0] = 300i16;
        accumulate_non_zero(&mut c, &mut d);
        for ci in c.iter() {
            assert!(*ci == 255);
        }
    }
    #[test]
    fn odd() {
        let mut a = [0u8; 3000];
        let mut b = [0i16; 3000];
        b[0] = 300i16;
        accumulate_odd(&mut a, &mut b);
        for ai in a.iter() {
            assert!(*ai == 212);
        }
    }
}
