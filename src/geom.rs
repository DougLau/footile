// geom.rs    Simple geometry stuff.
//
// Copyright (c) 2017-2018  Douglas P Lau
//
use std::f32;
use std::ops;

/// 2-dimensional vector.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

/// 2-dimensional vector with associated width.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2w {
    pub v: Vec2,
    pub w: f32,
}

/// An affine transform can translate, scale, rotate and skew 2D points.
///
/// A series of transforms can be combined into a single Transform struct.
/// This can be used by a [Plotter](struct.Plotter.html) to alter subsequent
/// points.
///
/// # Example
/// ```
/// use footile::Transform;
/// const PI: f32 = std::f32::consts::PI;
/// let t = Transform::new_translate(-50.0, -50.0)
///     .rotate(PI)
///     .translate(50.0, 50.0)
///     .scale(2.0, 2.0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    e: [f32; 6],
}

impl ops::Add for Vec2 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Vec2::new(self.x + other.x, self.y + other.y)
    }
}

impl ops::Sub for Vec2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Vec2::new(self.x - other.x, self.y - other.y)
    }
}

impl ops::Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, s: f32) -> Self {
        Vec2::new(self.x * s, self.y * s)
    }
}

impl ops::Mul for Vec2 {
    type Output = f32;

    /// Calculate the cross product of two Vec2
    fn mul(self, other: Self) -> f32 {
        self.x * other.y - self.y * other.x
    }
}

impl ops::Div<f32> for Vec2 {
    type Output = Self;

    fn div(self, s: f32) -> Self {
        Vec2::new(self.x / s, self.y / s)
    }
}

impl ops::Neg for Vec2 {
    type Output = Self;

    fn neg(self) -> Self {
        Vec2::new(-self.x, -self.y)
    }
}

impl Vec2 {
    /// Create a new Vec2
    pub fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }
    /// Create a zero Vec2
    pub fn zero() -> Self {
        Vec2::new(0.0, 0.0)
    }
    /// Get the magnitude of a Vec2
    pub fn mag(self) -> f32 {
        self.x.hypot(self.y)
    }
    /// Create a copy normalized to unit length
    pub fn normalize(self) -> Self {
        let m = self.mag();
        if m > 0.0 {
            self / m
        } else {
            Vec2::zero()
        }
    }
    /// Calculate the distance squared between two Vec2
    pub fn dist_sq(self, other: Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }
    /// Calculate the distance between two Vec2
    #[allow(dead_code)]
    pub fn dist(self, other: Self) -> f32 {
        self.dist_sq(other).sqrt()
    }
    /// Get the midpoint of two Vec2
    pub fn midpoint(self, other: Self) -> Self {
        let x = (self.x + other.x) / 2.0;
        let y = (self.y + other.y) / 2.0;
        Vec2::new(x, y)
    }
    /// Create a left-hand perpendicular Vec2
    pub fn left(self) -> Self {
        Vec2::new(-self.y, self.x)
    }
    /// Create a right-hand perpendicular Vec2
    #[allow(dead_code)]
    pub fn right(self) -> Self {
        Vec2::new(self.y, -self.x)
    }
    /// Calculate winding order for two Vec2.
    ///
    /// The Vec2 should be initialized as edges pointing toward the same vertex.
    /// Returns true if the winding order is widdershins (counter-clockwise).
    pub fn widdershins(self, other: Self) -> bool {
        // Cross product (with Z zero) is used to determine the winding order.
        (self.x * other.y) > (other.x * self.y)
    }
    /// Calculate linear interpolation of two Vec2
    ///
    /// * `t` Interpolation amount, from 0 to 1
    #[allow(dead_code)]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        let x = float_lerp(self.x, other.x, t);
        let y = float_lerp(self.y, other.y, t);
        Vec2::new(x, y)
    }
    /// Calculate the relative angle to another Vec2.
    ///
    /// The result will be between `-PI` and `+PI`.
    pub fn angle_rel(self, other: Self) -> f32 {
        const PI: f32 = f32::consts::PI;
        let th = self.y.atan2(self.x) - other.y.atan2(other.x);
        if th < -PI {
            th + 2.0 * PI
        } else if th > PI {
            th - 2.0 * PI
        } else {
            th
        }
    }
}

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
pub fn intersection(a0: Vec2, a1: Vec2, b0: Vec2, b1: Vec2) -> Option<Vec2> {
    let av = a0 - a1;
    let bv = b0 - b1;
    let den = av * bv;
    if den != 0.0 {
        let ca = a0 * a1;
        let cb = b0 * b1;
        let xn = bv.x * ca - av.x * cb;
        let yn = bv.y * ca - av.y * cb;
        Some(Vec2::new(xn / den, yn / den))
    } else {
        None
    }
}

impl Vec2w {
    /// Create a new Vec2w
    pub fn new(x: f32, y: f32, w: f32) -> Self {
        Vec2w {
            v: Vec2::new(x, y),
            w,
        }
    }
    /// Find the midpoint between two Vec2w
    pub fn midpoint(self, other: Self) -> Self {
        Vec2w {
            v: self.v.midpoint(other.v),
            w: (self.w + other.w) / 2.0,
        }
    }
}

impl ops::MulAssign for Transform {
    fn mul_assign(&mut self, other: Self) {
        self.e = self.mul_e(&other);
    }
}

impl ops::Mul for Transform {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        let e = self.mul_e(&other);
        Transform { e }
    }
}

impl ops::Mul<Vec2> for Transform {
    type Output = Vec2;

    fn mul(self, s: Vec2) -> Vec2 {
        let x = self.e[0] * s.x + self.e[1] * s.y + self.e[2];
        let y = self.e[3] * s.x + self.e[4] * s.y + self.e[5];
        Vec2::new(x, y)
    }
}

impl Transform {
    /// Create a new identity transform.
    pub fn new() -> Self {
        Transform {
            e: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
        }
    }
    /// Multiple two affine transforms.
    fn mul_e(&self, other: &Self) -> [f32; 6] {
        let mut e = [0.0; 6];
        e[0] = self.e[0] * other.e[0] + self.e[3] * other.e[1];
        e[1] = self.e[1] * other.e[0] + self.e[4] * other.e[1];
        e[2] = self.e[2] * other.e[0] + self.e[5] * other.e[1] + other.e[2];
        e[3] = self.e[0] * other.e[3] + self.e[3] * other.e[4];
        e[4] = self.e[1] * other.e[3] + self.e[4] * other.e[4];
        e[5] = self.e[2] * other.e[3] + self.e[5] * other.e[4] + other.e[5];
        e
    }
    /// Create a new translation transform.
    ///
    /// * `tx` Amount to translate X.
    /// * `ty` Amount to translate Y.
    pub fn new_translate(tx: f32, ty: f32) -> Self {
        Transform {
            e: [1.0, 0.0, tx, 0.0, 1.0, ty],
        }
    }
    /// Create a new scale transform.
    ///
    /// * `sx` Scale factor for X dimension.
    /// * `sy` Scale factor for Y dimension.
    pub fn new_scale(sx: f32, sy: f32) -> Self {
        Transform {
            e: [sx, 0.0, 0.0, 0.0, sy, 0.0],
        }
    }
    /// Create a new rotation transform.
    ///
    /// * `th` Angle to rotate coordinates (radians).
    pub fn new_rotate(th: f32) -> Self {
        let sn = th.sin();
        let cs = th.cos();
        Transform {
            e: [cs, -sn, 0.0, sn, cs, 0.0],
        }
    }
    /// Create a new skew transform.
    ///
    /// * `ax` Angle to skew X-axis (radians).
    /// * `ay` Angle to skew Y-axis (radians).
    pub fn new_skew(ax: f32, ay: f32) -> Self {
        let tnx = ax.tan();
        let tny = ay.tan();
        Transform {
            e: [1.0, tnx, 0.0, tny, 1.0, 0.0],
        }
    }
    /// Apply translation to a transform.
    ///
    /// * `tx` Amount to translate X.
    /// * `ty` Amount to translate Y.
    pub fn translate(mut self, tx: f32, ty: f32) -> Self {
        self *= Transform::new_translate(tx, ty);
        self
    }
    /// Apply scaling to a transform.
    ///
    /// * `sx` Scale factor for X dimension.
    /// * `sy` Scale factor for Y dimension.
    pub fn scale(mut self, sx: f32, sy: f32) -> Self {
        self *= Transform::new_scale(sx, sy);
        self
    }
    /// Apply rotation to a transform.
    ///
    /// * `th` Angle to rotate coordinates (radians).
    pub fn rotate(mut self, th: f32) -> Self {
        self *= Transform::new_rotate(th);
        self
    }
    /// Apply skew to a transform.
    ///
    /// * `ax` Angle to skew X-axis (radians).
    /// * `ay` Angle to skew Y-axis (radians).
    pub fn skew(mut self, ax: f32, ay: f32) -> Self {
        self *= Transform::new_skew(ax, ay);
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_vec2() {
        let a = Vec2::new(2.0, 1.0);
        let b = Vec2::new(3.0, 4.0);
        let c = Vec2::new(-1.0, 1.0);
        assert_eq!(a + b, Vec2::new(5.0, 5.0));
        assert_eq!(b - a, Vec2::new(1.0, 3.0));
        assert_eq!(a * 2.0, Vec2::new(4.0, 2.0));
        assert_eq!(a / 2.0, Vec2::new(1.0, 0.5));
        assert_eq!(-a, Vec2::new(-2.0, -1.0));
        assert_eq!(b.mag(), 5.0);
        assert_eq!(a.normalize(), Vec2::new(0.8944272, 0.4472136));
        assert_eq!(a.dist_sq(b), 10.0);
        assert_eq!(b.dist(Vec2::new(0.0, 0.0)), 5.0);
        assert_eq!(a.midpoint(b), Vec2::new(2.5, 2.5));
        assert_eq!(a.left(), Vec2::new(-1.0, 2.0));
        assert_eq!(a.right(), Vec2::new(1.0, -2.0));
        assert!(a.widdershins(b));
        assert!(!b.widdershins(a));
        assert!(b.widdershins(c));
        assert_eq!(a.angle_rel(b), -0.4636476);
        assert_eq!(c.angle_rel(Vec2::new(1.0, 1.0)), 1.5707963);
        assert_eq!(Vec2::new(-1.0, -1.0).angle_rel(c), 1.5707965);
    }
    #[test]
    fn test_identity() {
        assert_eq!(Transform::new().e, [1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
        assert_eq!(
            (Transform::new() * Transform::new()).e,
            [1.0, 0.0, 0.0, 0.0, 1.0, 0.0]
        );
        assert_eq!(Transform::new() * Vec2::new(1.0, 2.0), Vec2::new(1.0, 2.0));
    }
    #[test]
    fn test_translate() {
        assert_eq!(
            Transform::new_translate(1.5, -1.5).e,
            [1.0, 0.0, 1.5, 0.0, 1.0, -1.5]
        );
        assert_eq!(
            Transform::new().translate(2.5, -3.5).e,
            [1.0, 0.0, 2.5, 0.0, 1.0, -3.5]
        );
        assert_eq!(
            Transform::new().translate(5.0, 7.0) * Vec2::new(1.0, -2.0),
            Vec2::new(6.0, 5.0)
        );
    }
    #[test]
    fn test_scale() {
        assert_eq!(
            Transform::new_scale(2.0, 4.0).e,
            [2.0, 0.0, 0.0, 0.0, 4.0, 0.0]
        );
        assert_eq!(
            Transform::new().scale(3.0, 5.0).e,
            [3.0, 0.0, 0.0, 0.0, 5.0, 0.0]
        );
        assert_eq!(
            Transform::new().scale(2.0, 3.0) * Vec2::new(1.5, -2.0),
            Vec2::new(3.0, -6.0)
        );
    }
    #[test]
    fn test_rotate() {
        const PI: f32 = f32::consts::PI;
        const V: f32 = 0.00000008742278;
        assert_eq!(Transform::new_rotate(PI).e, [-1.0, V, 0.0, -V, -1.0, 0.0]);
        assert_eq!(
            Transform::new().rotate(PI).e,
            [-1.0, V, 0.0, -V, -1.0, 0.0]
        );
        assert_eq!(
            Transform::new().rotate(PI / 2.0) * Vec2::new(15.0, 7.0),
            Vec2::new(-7.0000005, 15.0)
        );
    }
    #[test]
    fn test_skew() {
        const PI: f32 = f32::consts::PI;
        assert_eq!(
            Transform::new_skew(PI / 2.0, 0.0).e,
            [1.0, -22877334.0, 0.0, 0.0, 1.0, 0.0]
        );
        assert_eq!(
            Transform::new().skew(PI / 2.0, 0.0).e,
            [1.0, -22877334.0, 0.0, 0.0, 1.0, 0.0]
        );
        assert_eq!(
            Transform::new_skew(0.0, PI / 4.0).e,
            [1.0, 0.0, 0.0, 1.0, 1.0, 0.0]
        );
        assert_eq!(
            Transform::new().skew(0.0, PI / 4.0).e,
            [1.0, 0.0, 0.0, 1.0, 1.0, 0.0]
        );
        assert_eq!(
            Transform::new().skew(0.0, PI / 4.0) * Vec2::new(5.0, 3.0),
            Vec2::new(5.0, 8.0)
        );
        assert_eq!(
            Transform::new().skew(0.0, PI / 4.0) * Vec2::new(15.0, 7.0),
            Vec2::new(15.0, 22.0)
        );
    }
    #[test]
    fn test_transform() {
        assert_eq!(
            (Transform::new_translate(1.0, 2.0)
                * Transform::new_scale(2.0, 2.0))
            .e,
            [2.0, 0.0, 2.0, 0.0, 2.0, 4.0]
        );
        assert_eq!(
            Transform::new_translate(3.0, 5.0)
                * Transform::new_scale(7.0, 11.0)
                * Transform::new_rotate(f32::consts::PI / 2.0)
                * Transform::new_skew(1.0, -2.0),
            Transform::new()
                .translate(3.0, 5.0)
                .scale(7.0, 11.0)
                .rotate(f32::consts::PI / 2.0)
                .skew(1.0, -2.0)
        );
    }
}
