// imgbuf.rs        Functions for blending image buffers.
//
// Copyright (c) 2017  Douglas P Lau
//
use libc::{c_uchar, c_int};

extern "C" {
    // LLVM can't auto-vectorize saturating add
    fn alpha_buf_saturating_add(dst: *mut c_uchar, src: *const c_uchar, len: c_int);
}

/// Compose two u8 buffers with saturating add.
#[allow(dead_code)]
pub(crate) fn alpha_saturating_add(dst: &mut [u8], src: &[u8]) {
    assert!(dst.len() == src.len());
    let w = dst.len() as i32;
    unsafe {
        alpha_buf_saturating_add(dst.as_mut_ptr(), src.as_ptr(), w);
    }
}

#[cfg(test)]
mod test {
    use super::alpha_saturating_add;
    #[test]
    fn saturating_add() {
        let mut a = [100u8; 3000];
        let mut b = [150u8; 3000];
        let c = [200u8; 3000];
        alpha_saturating_add(&mut a, &b);
        for ai in a.iter() {
            assert!(*ai == 250);
        }
        alpha_saturating_add(&mut b, &c);
        for bi in b.iter() {
            assert!(*bi == 255);
        }
    }
}
