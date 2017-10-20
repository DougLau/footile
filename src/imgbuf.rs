// imgbuf.rs        Functions for blending image buffers.
//
// Copyright (c) 2017  Douglas P Lau
//
use libc::{c_uchar, c_short, c_int};

extern "C" {
    fn accumulate_non_zero_c(dst: *mut c_uchar, src: *mut c_short, len: c_int);
    fn accumulate_odd_c(dst: *mut c_uchar, src: *mut c_short, len: c_int);
}

/// Accumulate sums over signed ares.
pub(crate) fn accumulate_non_zero(dst: &mut [u8], src: &mut [i16]) {
    assert!(dst.len() == src.len());
    let w = dst.len() as i32;
    unsafe {
        accumulate_non_zero_c(dst.as_mut_ptr(), src.as_mut_ptr(), w);
    }
}

/// Accumulate sums over signed ares.
pub(crate) fn accumulate_odd(dst: &mut [u8], src: &mut [i16]) {
    assert!(dst.len() == src.len());
    let w = dst.len() as i32;
    unsafe {
        accumulate_odd_c(dst.as_mut_ptr(), src.as_mut_ptr(), w);
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
