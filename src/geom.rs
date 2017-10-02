// geom.rs    Simple geometry stuff.
//
// Copyright (c) 2017  Douglas P Lau
//
use std::cmp;
use std::fmt;
use std::f32;
use std::ops;

// Declare floating point type to use
type Float = f32;

/// 2-dimensional vector
#[derive(Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: Float,
    pub y: Float,
}

/// 3-dimensional vector
#[derive(Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: Float,
    pub y: Float,
    pub z: Float,
}

/// 3-dimensional vector
#[derive(Clone, Copy, PartialEq)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Pos trait allows point lookup by handle
pub trait Pos {
    fn pos(&self, hnd: u32) -> Vec3i;
}

/// Bounding box
#[derive(Clone, Copy)]
pub struct BBox {
    pub center: Vec3i,
    pub half_len: i32,
}

impl fmt::Debug for Vec2 {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "({},{})", self.x, self.y)
	}
}

impl ops::Add for Vec2 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Vec2::new(self.x + other.x, self.y + other.y)
    }
}

impl ops::Sub for Vec2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self{
        Vec2::new(self.x - other.x, self.y - other.y)
    }
}

impl ops::Mul<Float> for Vec2 {
    type Output = Self;

    fn mul(self, s: Float) -> Self {
        Vec2::new(self.x * s, self.y * s)
    }
}

impl ops::Mul for Vec2 {
    type Output = Float;

    /// Calculate the cross product of two Vec2
    fn mul(self, other: Self) -> Float {
        self.x * other.y - self.y * other.x
    }
}

impl ops::Div<Float> for Vec2 {
    type Output = Self;

    fn div(self, s: Float) -> Self {
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
    pub fn new(x: Float, y: Float) -> Self {
        Vec2 { x: x, y: y }
    }
    /// Create a zero Vec2
    pub fn zero() -> Self {
        Vec2::new(0 as Float, 0 as Float)
    }
    /// Get the magnitude of a Vec2
    pub fn mag(self) -> Float {
        self.x.hypot(self.y)
    }
    /// Create a copy normalized to unit length
    pub fn normalize(self) -> Self {
        let m = self.mag();
        if m > 0 as Float {
            self / m
        } else {
            Vec2::zero()
        }
    }
    /// Calculate the distance squared between two Vec2
    pub fn dist_sq(self, other: Self) -> Float {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }
    /// Calculate the distance between two Vec2
    pub fn dist(self, other: Self) -> Float {
        self.dist_sq(other).sqrt()
    }
    /// Get the midpoint of two Vec2
    pub fn midpoint(self, other: Self) -> Self {
        let x = (self.x + other.x) / 2 as Float;
        let y = (self.y + other.y) / 2 as Float;
        Vec2::new(x, y)
    }
    /// Create a left-hand perpendicular Vec2
    pub fn left(self) -> Self {
        Vec2::new(-self.y, self.x)
    }
    /// Create a right-hand perpendicular Vec2
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
    pub fn lerp(self, other: Self, t: Float) -> Self {
        let x = float_lerp(self.x, other.x, t);
        let y = float_lerp(self.y, other.y, t);
        Vec2::new(x, y)
    }
    /// Calculate the relative angle to another Vec2.
    ///
    /// The result will be between `-PI` and `+PI`.
    pub fn angle_rel(self, other: Self) -> Float {
        let pi = f32::consts::PI;
        let th = self.y.atan2(self.x) - other.y.atan2(other.x);
        if th < -pi {
            th + 2f32 * pi
        } else if th > pi {
            th - 2f32 * pi
        } else {
            th
        }
    }
}

/// Calculate linear interpolation of two values
///
/// The t value should be between 0 and 1.
pub fn float_lerp(a: Float, b: Float, t: Float) -> Float {
    b + (a - b) * t
}

/// Calculate intersection point of two lines.
///
/// * `a0` First point on line a.
/// * `a1` Second point on line a.
/// * `b0` First point on line b.
/// * `b1` Second point on line b.
/// Returns None if the lines are colinear.
pub fn intersection(a0: Vec2,
                    a1: Vec2,
                    b0: Vec2,
                    b1: Vec2) -> Option<Vec2>
{
    let av = a0 - a1;
    let bv = b0 - b1;
    let den = av * bv;
    if den != 0 as Float {
        let ca = a0 * a1;
        let cb = b0 * b1;
        let xn = bv.x * ca - av.x * cb;
        let yn = bv.y * ca - av.y * cb;
        Some(Vec2::new(xn / den, yn / den))
    } else {
        None
    }
}

impl fmt::Debug for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{},{})", self.x, self.y, self.z)
    }
}

impl Vec3 {
    /// Create a new Vec3
    pub fn new(x: Float, y: Float, z: Float) -> Self {
        Vec3 { x: x, y: y, z: z }
    }
    /// Find the midpoint between two Vec3
    pub fn midpoint(self, other: Self) -> Self {
        let x = (self.x + other.x) / 2 as Float;
        let y = (self.y + other.y) / 2 as Float;
        let z = (self.z + other.z) / 2 as Float;
        Vec3::new(x, y, z)
    }
}

impl fmt::Debug for Vec3i {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{},{})", self.x, self.y, self.z)
    }
}

impl Vec3i {
    /// Create a new Vec3i
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Vec3i { x: x, y: y, z: z }
    }
    /// Find the minimum ordinal value
    fn min_p(self) -> i32 {
        cmp::min(cmp::min(self.x, self.y), self.z)
    }
    /// Find the maximum ordinal value
    fn max_p(self) -> i32 {
        cmp::max(cmp::max(self.x, self.y), self.z)
    }
    /// Calculate the distance squared between two Vec3i
    pub fn dist_sq(self, other: Self) -> i32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        let dz = other.z - self.z;
        dx * dx + dy * dy + dz * dz
    }
}

impl fmt::Debug for BBox {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}±{}", self.center, self.half_len)
    }
}

impl BBox {
    pub fn empty() -> BBox {
        BBox { center: Vec3i::new(0, 0, 0), half_len: -1 }
    }
    pub fn new(p: Vec3i) -> BBox {
        BBox { center: p, half_len: 1 }
    }
    fn min_p(&self) -> i32 {
        if self.half_len > 0 {
            self.center.min_p() - self.half_len
        } else {
            self.center.min_p()
        }
    }
    fn max_p(&self) -> i32 {
        if self.half_len > 0 {
            self.center.max_p() + self.half_len
        } else {
            self.center.max_p()
        }
    }
    pub fn extend(&mut self, p: Vec3i) {
        self.center = self.move_center(p);
        self.half_len *= 2;
    }
    fn move_center(&self, p: Vec3i) -> Vec3i {
        let min_p = self.min_p();
        if p.min_p() < min_p {
            return Vec3i::new(min_p, min_p, min_p);
        } else {
            let max_p = self.max_p();
            return Vec3i::new(max_p, max_p, max_p);
        }
    }
    pub fn contains(&self, p: Vec3i) -> bool {
        let Vec3i { x, y, z } = self.center;
        let hl = self.half_len;
        (p.x >= x - hl) &&
        (p.x <  x + hl) &&
        (p.y >= y - hl) &&
        (p.y <  y + hl) &&
        (p.z >= z - hl) &&
        (p.z <  z + hl)
    }
}

#[test]
fn test_vec2() {
    let a = Vec2::new(2f32, 1f32);
    let b = Vec2::new(3f32, 4f32);
    let c = Vec2::new(-1f32, 1f32);
    assert!(a + b == Vec2::new(5f32, 5f32));
    assert!(b - a == Vec2::new(1f32, 3f32));
    assert!(a * 2f32 == Vec2::new(4f32, 2f32));
    assert!(a / 2f32 == Vec2::new(1f32, 0.5f32));
    assert!(-a == Vec2::new(-2f32, -1f32));
    assert!(b.mag() == 5f32);
    assert!(a.normalize() == Vec2::new(0.8944272f32, 0.4472136f32));
    assert!(a.dist_sq(b) == 10f32);
    assert!(b.dist(Vec2::new(0f32, 0f32)) == 5f32);
    assert!(a.midpoint(b) == Vec2::new(2.5f32, 2.5f32));
    assert!(a.left() == Vec2::new(-1f32, 2f32));
    assert!(a.right() == Vec2::new(1f32, -2f32));
    assert!(a.angle_rel(b) == -0.4636476f32);
    assert!(c.angle_rel(Vec2::new(1f32, 1f32)) == 1.5707963f32);
    assert!(Vec2::new(-1f32, -1f32).angle_rel(c) == 1.5707965f32);
}

#[test]
fn test_bbox() {
    let v0 = Vec3i::new(0, 0, 0);
    let v1 = Vec3i::new(1, 1, 1);
    let v2 = Vec3i::new(2, 2, 2);
    let v3 = Vec3i::new(3, 3, 3);
    let v4 = Vec3i::new(4, 4, 4);
    let v5 = Vec3i::new(5, 5, 5);
    let v6 = Vec3i::new(6, 6, 6);
    let v8 = Vec3i::new(8, 8, 8);
    let vn1 = Vec3i::new(-1, -1, -1);
    let mut b = BBox::new(v2);
    assert!(b.contains(v2));
    b.extend(v0);
    assert!(b.center == v1);
    assert!(b.half_len == 2);
    assert!(b.contains(v1));
    assert!(b.contains(v2));
    b.extend(v4);
    assert!(b.center == v3);
    assert!(b.half_len == 4);
    assert!(b.contains(v0));
    assert!(b.contains(v1));
    assert!(b.contains(v2));
    assert!(b.contains(v3));
    //
    //    0 1 2 3 4 5 6 7 8
    //   .       .___.   .
    //   .       ._______.
    //   ._______________.
    b = BBox::new(v5);
    b.extend(v8);
    assert!(b.center == v6);
    assert!(b.half_len == 2);
    assert!(b.contains(v4));
    assert!(b.contains(v5));
    assert!(b.contains(v6));
    b.extend(v0);
    assert!(b.center == v4);
    assert!(b.half_len == 4);
    assert!(b.contains(v5));
    b.extend(v8);
    assert!(b.center == v8);
    assert!(b.half_len == 8);
    assert!(b.contains(v6));
    b.extend(vn1);
    assert!(b.center == v0);
    assert!(b.half_len == 16);

    // test negative point
    b = BBox::new(vn1);
    b.extend(v0);
    assert!(b.center == v0);
    assert!(b.half_len == 2);
}
