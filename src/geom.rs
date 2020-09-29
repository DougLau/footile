// geom.rs    Simple geometry stuff.
//
// Copyright (c) 2017-2020  Douglas P Lau
//
use pointy::Pt32;

/// 2-dimensional vector / point with associated width.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WidePt(pub Pt32, pub f32);

/// Calculate linear interpolation of two values
///
/// The t value should be between 0 and 1.
pub fn float_lerp(a: f32, b: f32, t: f32) -> f32 {
    b + (a - b) * t
}

/// Calculate intersection point of two lines.
///
/// * `a0` First point on line a.
/// * `a1` Second point on line a.
/// * `b0` First point on line b.
/// * `b1` Second point on line b.
/// Returns None if the lines are colinear.
pub fn intersection(a0: Pt32, a1: Pt32, b0: Pt32, b1: Pt32) -> Option<Pt32> {
    let av = a0 - a1;
    let bv = b0 - b1;
    let den = av * bv;
    if den != 0.0 {
        let ca = a0 * a1;
        let cb = b0 * b1;
        let xn = bv.x() * ca - av.x() * cb;
        let yn = bv.y() * ca - av.y() * cb;
        Some(Pt32(xn / den, yn / den))
    } else {
        None
    }
}

impl Default for WidePt {
    fn default() -> Self {
        WidePt(Pt32::default(), 1.0)
    }
}

impl WidePt {
    /// Get the width
    pub fn w(self) -> f32 {
        self.1
    }

    /// Find the midpoint between two wide points
    pub fn midpoint(self, rhs: Self) -> Self {
        let v = self.0.midpoint(rhs.0);
        let w = (self.w() + rhs.w()) / 2.0;
        WidePt(v, w)
    }
}
