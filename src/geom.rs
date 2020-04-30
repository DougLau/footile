// geom.rs    Simple geometry stuff.
//
// Copyright (c) 2017-2020  Douglas P Lau
//
use std::f32;
use std::ops::{Add, Div, Mul, MulAssign, Neg, Sub};

/// 2-dimensional vector / point.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pt(pub f32, pub f32);

/// 2-dimensional vector / point with associated width.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WidePt(pub Pt, pub f32);

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
    /// First six values in 3x3 matrix (last row assumed to be 0 0 1)
    e: [f32; 6],
}

impl Add for Pt {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Pt(self.x() + rhs.x(), self.y() + rhs.y())
    }
}

impl Sub for Pt {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Pt(self.x() - rhs.x(), self.y() - rhs.y())
    }
}

impl Mul<f32> for Pt {
    type Output = Self;

    fn mul(self, s: f32) -> Self {
        Pt(self.x() * s, self.y() * s)
    }
}

impl Mul for Pt {
    type Output = f32;

    /// Calculate the cross product of two vectors
    fn mul(self, rhs: Self) -> f32 {
        self.x() * rhs.y() - self.y() * rhs.x()
    }
}

impl Div<f32> for Pt {
    type Output = Self;

    fn div(self, s: f32) -> Self {
        Pt(self.x() / s, self.y() / s)
    }
}

impl Neg for Pt {
    type Output = Self;

    fn neg(self) -> Self {
        Pt(-self.x(), -self.y())
    }
}

impl Pt {
    /// Get the X value
    pub fn x(self) -> f32 {
        self.0
    }

    /// Get the Y value
    pub fn y(self) -> f32 {
        self.1
    }

    /// Get the magnitude of a vector
    pub fn mag(self) -> f32 {
        self.x().hypot(self.y())
    }

    /// Create a copy normalized to unit length
    pub fn normalize(self) -> Self {
        let m = self.mag();
        if m > 0.0 {
            self / m
        } else {
            Pt::default()
        }
    }

    /// Calculate the distance squared between two points
    pub fn dist_sq(self, rhs: Self) -> f32 {
        let dx = self.x() - rhs.x();
        let dy = self.y() - rhs.y();
        dx * dx + dy * dy
    }

    /// Calculate the distance between two points
    #[allow(dead_code)]
    pub fn dist(self, rhs: Self) -> f32 {
        self.dist_sq(rhs).sqrt()
    }

    /// Get the midpoint of two points
    pub fn midpoint(self, rhs: Self) -> Self {
        let x = (self.x() + rhs.x()) / 2.0;
        let y = (self.y() + rhs.y()) / 2.0;
        Pt(x, y)
    }

    /// Create a left-hand perpendicular vector
    pub fn left(self) -> Self {
        Pt(-self.y(), self.x())
    }

    /// Create a right-hand perpendicular vector
    #[allow(dead_code)]
    pub fn right(self) -> Self {
        Pt(self.y(), -self.x())
    }

    /// Calculate winding order for two points.
    ///
    /// The points should be initialized as edges pointing toward the same
    /// vertex.
    /// Returns true if the winding order is widdershins (counter-clockwise).
    pub fn widdershins(self, rhs: Self) -> bool {
        // Cross product (with Z zero) is used to determine the winding order.
        (self.x() * rhs.y()) > (rhs.x() * self.y())
    }

    /// Calculate linear interpolation of two points.
    ///
    /// * `t` Interpolation amount, from 0 to 1
    #[allow(dead_code)]
    pub fn lerp(self, rhs: Self, t: f32) -> Self {
        let x = float_lerp(self.x(), rhs.x(), t);
        let y = float_lerp(self.y(), rhs.y(), t);
        Pt(x, y)
    }

    /// Calculate the relative angle to another vector / point.
    ///
    /// The result will be between `-PI` and `+PI`.
    pub fn angle_rel(self, rhs: Self) -> f32 {
        const PI: f32 = f32::consts::PI;
        let th = self.y().atan2(self.x()) - rhs.y().atan2(rhs.x());
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
pub fn intersection(a0: Pt, a1: Pt, b0: Pt, b1: Pt) -> Option<Pt> {
    let av = a0 - a1;
    let bv = b0 - b1;
    let den = av * bv;
    if den != 0.0 {
        let ca = a0 * a1;
        let cb = b0 * b1;
        let xn = bv.x() * ca - av.x() * cb;
        let yn = bv.y() * ca - av.y() * cb;
        Some(Pt(xn / den, yn / den))
    } else {
        None
    }
}

impl Default for WidePt {
    fn default() -> Self {
        WidePt(Pt::default(), 1.0)
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

impl MulAssign for Transform {
    fn mul_assign(&mut self, rhs: Self) {
        self.e = self.mul_e(&rhs);
    }
}

impl Mul for Transform {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let e = self.mul_e(&rhs);
        Transform { e }
    }
}

impl Mul<Pt> for Transform {
    type Output = Pt;

    fn mul(self, s: Pt) -> Pt {
        let x = self.e[0] * s.x() + self.e[1] * s.y() + self.e[2];
        let y = self.e[3] * s.x() + self.e[4] * s.y() + self.e[5];
        Pt(x, y)
    }
}

impl Default for Transform {
    /// Create a new identity transform.
    fn default() -> Self {
        Transform {
            e: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
        }
    }
}

impl Transform {
    /// Multiple two affine transforms.
    fn mul_e(&self, rhs: &Self) -> [f32; 6] {
        let mut e = [0.0; 6];
        e[0] = self.e[0] * rhs.e[0] + self.e[3] * rhs.e[1];
        e[1] = self.e[1] * rhs.e[0] + self.e[4] * rhs.e[1];
        e[2] = self.e[2] * rhs.e[0] + self.e[5] * rhs.e[1] + rhs.e[2];
        e[3] = self.e[0] * rhs.e[3] + self.e[3] * rhs.e[4];
        e[4] = self.e[1] * rhs.e[3] + self.e[4] * rhs.e[4];
        e[5] = self.e[2] * rhs.e[3] + self.e[5] * rhs.e[4] + rhs.e[5];
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
    fn test_pt() {
        let a = Pt(2.0, 1.0);
        let b = Pt(3.0, 4.0);
        let c = Pt(-1.0, 1.0);
        assert_eq!(a + b, Pt(5.0, 5.0));
        assert_eq!(b - a, Pt(1.0, 3.0));
        assert_eq!(a * 2.0, Pt(4.0, 2.0));
        assert_eq!(a / 2.0, Pt(1.0, 0.5));
        assert_eq!(-a, Pt(-2.0, -1.0));
        assert_eq!(b.mag(), 5.0);
        assert_eq!(a.normalize(), Pt(0.8944272, 0.4472136));
        assert_eq!(a.dist_sq(b), 10.0);
        assert_eq!(b.dist(Pt(0.0, 0.0)), 5.0);
        assert_eq!(a.midpoint(b), Pt(2.5, 2.5));
        assert_eq!(a.left(), Pt(-1.0, 2.0));
        assert_eq!(a.right(), Pt(1.0, -2.0));
        assert!(a.widdershins(b));
        assert!(!b.widdershins(a));
        assert!(b.widdershins(c));
        assert_eq!(a.angle_rel(b), -0.4636476);
        assert_eq!(c.angle_rel(Pt(1.0, 1.0)), 1.5707963);
        assert_eq!(Pt(-1.0, -1.0).angle_rel(c), 1.5707965);
    }

    #[test]
    fn test_identity() {
        assert_eq!(Transform::default().e, [1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
        assert_eq!(
            (Transform::default() * Transform::default()).e,
            [1.0, 0.0, 0.0, 0.0, 1.0, 0.0]
        );
        assert_eq!(
            Transform::default() * Pt(1.0, 2.0),
            Pt(1.0, 2.0)
        );
    }

    #[test]
    fn test_translate() {
        assert_eq!(
            Transform::new_translate(1.5, -1.5).e,
            [1.0, 0.0, 1.5, 0.0, 1.0, -1.5]
        );
        assert_eq!(
            Transform::default().translate(2.5, -3.5).e,
            [1.0, 0.0, 2.5, 0.0, 1.0, -3.5]
        );
        assert_eq!(
            Transform::default().translate(5.0, 7.0) * Pt(1.0, -2.0),
            Pt(6.0, 5.0)
        );
    }

    #[test]
    fn test_scale() {
        assert_eq!(
            Transform::new_scale(2.0, 4.0).e,
            [2.0, 0.0, 0.0, 0.0, 4.0, 0.0]
        );
        assert_eq!(
            Transform::default().scale(3.0, 5.0).e,
            [3.0, 0.0, 0.0, 0.0, 5.0, 0.0]
        );
        assert_eq!(
            Transform::default().scale(2.0, 3.0) * Pt(1.5, -2.0),
            Pt(3.0, -6.0)
        );
    }

    #[test]
    fn test_rotate() {
        const PI: f32 = f32::consts::PI;
        const V: f32 = 0.00000008742278;
        assert_eq!(Transform::new_rotate(PI).e, [-1.0, V, 0.0, -V, -1.0, 0.0]);
        assert_eq!(
            Transform::default().rotate(PI).e,
            [-1.0, V, 0.0, -V, -1.0, 0.0]
        );
        assert_eq!(
            Transform::default().rotate(PI / 2.0) * Pt(15.0, 7.0),
            Pt(-7.0000005, 15.0)
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
            Transform::default().skew(PI / 2.0, 0.0).e,
            [1.0, -22877334.0, 0.0, 0.0, 1.0, 0.0]
        );
        assert_eq!(
            Transform::new_skew(0.0, PI / 4.0).e,
            [1.0, 0.0, 0.0, 1.0, 1.0, 0.0]
        );
        assert_eq!(
            Transform::default().skew(0.0, PI / 4.0).e,
            [1.0, 0.0, 0.0, 1.0, 1.0, 0.0]
        );
        assert_eq!(
            Transform::default().skew(0.0, PI / 4.0) * Pt(5.0, 3.0),
            Pt(5.0, 8.0)
        );
        assert_eq!(
            Transform::default().skew(0.0, PI / 4.0) * Pt(15.0, 7.0),
            Pt(15.0, 22.0)
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
            Transform::default()
                .translate(3.0, 5.0)
                .scale(7.0, 11.0)
                .rotate(f32::consts::PI / 2.0)
                .skew(1.0, -2.0)
        );
    }
}
