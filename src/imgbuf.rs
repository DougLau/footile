// imgbuf.rs        Functions for blending image buffers.
//
// Copyright (c) 2017  Douglas P Lau
//

/// Accumulate sums over signed ares.
pub(crate) fn accumulate_non_zero(dst: &mut [u8], src: &mut [i16]) {
    assert!(dst.len() == src.len());
    let w = dst.len() as isize;
    unsafe {
        accumulate_non_zero_impl(dst.as_mut_ptr(), src.as_mut_ptr(), w);
    }
}

/// Accumulate sums over signed ares.
pub(crate) fn accumulate_odd(dst: &mut [u8], src: &mut [i16]) {
    assert!(dst.len() == src.len());
    let w = dst.len() as isize;
    unsafe {
        accumulate_odd_impl(dst.as_mut_ptr(), src.as_mut_ptr(), w);
    }
}

/* Accumulate signed area buffer and store in dest buffer.
 * Source buffer is zeroed upon return.
 *
 * dst: Destination buffer.
 * src: Source buffer.
 * len: Size of buffers.
 */
#[cfg(
    not(
        all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "ssse3"
        )
    )
)]
pub unsafe fn accumulate_non_zero_impl(dst: *mut u8, src: *mut i16, len: isize) {
    let mut i = 0;
    let mut s: i16 = 0;
    while i < len {
        s += *src.offset(i);
        *src.offset(i) = 0;
        *dst.offset(i) = s.max(0).min(255) as u8;
        i += 1;
    }
}
/* Accumulate signed area buffer and store in dest buffer.
 * Source buffer is zeroed upon return.
 *
 * dst: Destination buffer.
 * src: Source buffer.
 * len: Size of buffers.
 */
#[cfg(
    not(
        all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "ssse3"
        )
    )
)]
pub unsafe fn accumulate_odd_impl(dst: *mut u8, src: *mut i16, len: isize) {
    let mut i = 0;
    let mut s: i16 = 0;
    while i < len {
        s += *src.offset(i);
        *src.offset(i) = 0;
        *dst.offset(i) = if s & 0x100 != 0 {
            if (s & 0xff) == 0 {
                0xff
            } else {
                0x100 - (s & 0xff)
            }
        } else {
            s & 0xff
        } as u8;
        i += 1;
    }
}

#[cfg(target_arch = "x86")]
pub use std::arch::x86::{
    __m128i, _mm_abs_epi16, _mm_add_epi16, _mm_and_si128, _mm_loadu_si128, _mm_packus_epi16,
    _mm_set1_epi16, _mm_setzero_si128, _mm_shuffle_epi8, _mm_slli_si128, _mm_storel_epi64,
    _mm_storeu_si128, _mm_sub_epi16,
};
#[cfg(target_arch = "x86_64")]
pub use std::arch::x86_64::{
    __m128i, _mm_abs_epi16, _mm_add_epi16, _mm_and_si128, _mm_loadu_si128, _mm_packus_epi16,
    _mm_set1_epi16, _mm_setzero_si128, _mm_shuffle_epi8, _mm_slli_si128, _mm_storel_epi64,
    _mm_storeu_si128, _mm_sub_epi16,
};

/* Accumulate signed area buffer and store in dest buffer.
 * Source buffer is zeroed upon return.
 *
 * dst: Destination buffer.
 * src: Source buffer.
 * len: Size of buffers.
 */
#[cfg(
    all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "ssse3"
    )
)]
pub unsafe fn accumulate_non_zero_impl(dst: *mut u8, src: *mut i16, len: isize) {
    let mut i = 0;
    let zero = _mm_setzero_si128();
    let mut sum: __m128i = zero;
    /* mask for shuffling final sum into all lanes */
    let mask = _mm_set1_epi16(0xf0e);
    loop {
        let mut a: __m128i = _mm_loadu_si128(src.offset(i) as *const __m128i);
        /* zeroing now is faster than memset later */
        _mm_storeu_si128(src.offset(i) as *mut __m128i, zero);
        /*   a7 a6 a5 a4 a3 a2 a1 a0 */
        /* + a3 a2 a1 a0 __ __ __ __ */
        a = _mm_add_epi16(a, _mm_slli_si128(a, 8));
        /* + a5 a4 a3 a2 a1 a0 __ __ */
        /* + a1 a0 __ __ __ __ __ __ */
        a = _mm_add_epi16(a, _mm_slli_si128(a, 4));
        /* + a6 a5 a4 a3 a2 a1 a0 __ */
        /* + a2 a1 a0 __ __ __ __ __ */
        /* + a4 a3 a2 a1 a0 __ __ __ */
        /* + a0 __ __ __ __ __ __ __ */
        a = _mm_add_epi16(a, _mm_slli_si128(a, 2));
        a = _mm_add_epi16(a, sum);
        let b = _mm_packus_epi16(a, a);
        _mm_storel_epi64(dst.offset(i) as *mut __m128i, b);
        i += 8;
        /* breaking here saves one shuffle */
        if i >= len {
            break;
        }
        sum = _mm_shuffle_epi8(a, mask)
    }
}

/* Accumulate signed area buffer and store in dest buffer.
 * Source buffer is zeroed upon return.
 *
 * dst: Destination buffer.
 * src: Source buffer.
 * len: Size of buffers.
 */
#[cfg(
    all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "ssse3"
    )
)]
pub unsafe fn accumulate_odd_impl(dst: *mut u8, src: *mut i16, len: isize) {
    let mut i = 0;
    let zero = _mm_setzero_si128();
    let mut sum: __m128i = zero;
    /* mask for shuffling final sum into all lanes */
    let mask = _mm_set1_epi16(0xf0e);
    loop {
        let mut a: __m128i = _mm_loadu_si128(src.offset(i) as *const __m128i);
        /* zeroing now is faster than memset later */
        _mm_storeu_si128(src.offset(i) as *mut __m128i, zero);
        /*   a7 a6 a5 a4 a3 a2 a1 a0 */
        /* + a3 a2 a1 a0 __ __ __ __ */
        a = _mm_add_epi16(a, _mm_slli_si128(a, 8));
        /* + a5 a4 a3 a2 a1 a0 __ __ */
        /* + a1 a0 __ __ __ __ __ __ */
        a = _mm_add_epi16(a, _mm_slli_si128(a, 4));
        /* + a6 a5 a4 a3 a2 a1 a0 __ */
        /* + a2 a1 a0 __ __ __ __ __ */
        /* + a4 a3 a2 a1 a0 __ __ __ */
        /* + a0 __ __ __ __ __ __ __ */
        a = _mm_add_epi16(a, _mm_slli_si128(a, 2));
        a = _mm_add_epi16(a, sum);
        let mut c = _mm_and_si128(a, _mm_set1_epi16(0xff));
        let d = _mm_and_si128(a, _mm_set1_epi16(0x100));
        c = _mm_sub_epi16(c, d);
        c = _mm_abs_epi16(c);
        let b = _mm_packus_epi16(c, c);
        _mm_storel_epi64(dst.offset(i) as *mut __m128i, b);
        i += 8;
        /* breaking here saves one shuffle */
        if i >= len {
            break;
        }
        sum = _mm_shuffle_epi8(a, mask)
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
