// plotter.rs      Vector path plotter.
//
// Copyright (c) 2017-2019  Douglas P Lau
//
use crate::fig::Fig;
use crate::geom::{Transform, Vec2, Vec2w, float_lerp};
use crate::path::{FillRule, PathOp};
use crate::stroker::{JoinStyle, Stroke};
use pix::{Mask8, Raster, RasterBuilder};
use std::borrow::Borrow;

/// Plotter for 2D vector paths.
///
/// This is a software vector rasterizer featuring anti-aliasing.
/// Paths can be created using [PathBuilder](struct.PathBuilder.html).
/// The plotter contains a mask of the current plot, which is affected by fill
/// and stroke calls.
///
/// # Example
/// ```
/// use footile::{PathBuilder, Plotter};
/// let path = PathBuilder::new().pen_width(3.0)
///                        .move_to(50.0, 34.0)
///                        .cubic_to(4.0, 16.0, 16.0, 28.0, 0.0, 32.0)
///                        .cubic_to(-16.0, -4.0, -4.0, -16.0, 0.0, -32.0)
///                        .close().build();
/// let mut p = Plotter::new(100, 100);
/// p.stroke(&path);
/// ```
pub struct Plotter {
    mask       : Raster<Mask8>,     // image mask
    sgn_area   : Vec<i16>,          // signed area buffer
    pen        : Vec2w,             // current pen position and width
    transform  : Transform,         // user to pixel affine transform
    tol_sq     : f32,               // curve decomposition tolerance squared
    s_width    : f32,               // current stroke width
    join_style : JoinStyle,         // current join style
}

/// Plot destination
trait PlotDest {
    /// Add a point.
    ///
    /// * `pt` Point to add (w indicates stroke width).
    fn add_point(&mut self, pt: Vec2w);
    /// Close the current sub-figure.
    ///
    /// * `joined` If true, join ends of sub-plot.
    fn close(&mut self, joined: bool);
}

impl PlotDest for Fig {
    fn add_point(&mut self, pt: Vec2w) {
        Fig::add_point(self, pt.v);
    }
    fn close(&mut self, _joined: bool) {
        Fig::close(self);
    }
}

impl PlotDest for Stroke {
    fn add_point(&mut self, pt: Vec2w) {
        Stroke::add_point(self, pt);
    }
    fn close(&mut self, joined: bool) {
        Stroke::close(self, joined);
    }
}

impl Plotter {
    /// Create a new plotter.
    ///
    /// * `width` Width in pixels.
    /// * `height` Height in pixels.
    pub fn new(width: u32, height: u32) -> Plotter {
        let tol = 0.3;
        let w = if width > 0 { width } else { 100 };
        let h = if height > 0 { height } else { 100 };
        let len = w as usize;
        // Capacity must be 8-element multiple (for SIMD)
        let cap = ((len + 7) >> 3) << 3;
        let mut sgn_area = vec![0i16; cap];
        // Remove excess elements
        for _ in 0..cap-len { sgn_area.pop(); };
        Plotter {
            mask       : RasterBuilder::new().with_clear(w, h),
            sgn_area   : sgn_area,
            pen        : Vec2w::new(0.0, 0.0, 1.0),
            transform  : Transform::new(),
            tol_sq     : tol * tol,
            s_width    : 1.0,
            join_style : JoinStyle::Miter(4.0),
        }
    }
    /// Get width in pixels.
    pub fn width(&self) -> u32 {
        self.mask.width()
    }
    /// Get height in pixels.
    pub fn height(&self) -> u32 {
        self.mask.height()
    }
    /// Reset pen.
    fn reset(&mut self) {
        self.pen = Vec2w::new(0.0, 0.0, self.s_width);
    }
    /// Clear the mask.
    pub fn clear_mask(&mut self) -> &mut Self {
        self.mask.clear();
        self
    }
    /// Set tolerance threshold for curve decomposition.
    pub fn set_tolerance(&mut self, t: f32) -> &mut Self {
        let tol = t.max(0.01);
        self.tol_sq = tol * tol;
        self
    }
    /// Set the transform.
    pub fn set_transform(&mut self, t: Transform) -> &mut Self {
        self.transform = t;
        self
    }
    /// Set pen stroke width.
    ///
    /// All subsequent path points will be affected, until the stroke width
    /// is changed again.
    ///
    /// * `width` Pen stroke width.
    fn pen_width(&mut self, width: f32) {
        self.s_width = width;
    }
    /// Set stroke join style.
    ///
    /// * `js` Join style.
    pub fn set_join(&mut self, js: JoinStyle) -> &mut Self {
        self.join_style = js;
        self
    }
    /// Move the pen.
    fn move_pen(&mut self, p: Vec2w) {
        self.pen = p;
    }
    /// Transform a point.
    fn transform_point(&self, p: Vec2w) -> Vec2w {
        let pt = self.transform * p.v;
        Vec2w::new(pt.x, pt.y, p.w)
    }
    /// Add a series of ops.
    fn add_ops<'a, T, D>(&mut self, ops: T, dst: &mut D)
        where T: IntoIterator, T::Item: Borrow<PathOp>, D: PlotDest
    {
        self.reset();
        for op in ops {
            self.add_op(dst, op.borrow());
        }
    }
    /// Add a path operation.
    fn add_op<D: PlotDest>(&mut self, dst: &mut D, op: &PathOp) {
        match op {
            &PathOp::Close()                 => self.close(dst),
            &PathOp::Move(bx, by)            => self.move_to(dst, bx, by),
            &PathOp::Line(bx, by)            => self.line_to(dst, bx, by),
            &PathOp::Quad(bx, by, cx, cy)    => self.quad_to(dst, bx,by,cx,cy),
            &PathOp::Cubic(bx,by,cx,cy,dx,dy)=> self.cubic_to(dst, bx, by, cx,
                                                              cy, dx, dy),
            &PathOp::PenWidth(w)             => self.pen_width(w),
        };
    }
    /// Close current sub-path and move pen to origin.
    fn close<D: PlotDest>(&mut self, dst: &mut D) {
        dst.close(true);
        self.reset();
    }
    /// Move pen to a point.
    ///
    /// * `bx` X-position of point.
    /// * `by` Y-position of point.
    fn move_to<D: PlotDest>(&mut self, dst: &mut D, bx: f32, by: f32) {
        let p = Vec2w::new(bx, by, self.s_width);
        dst.close(false);
        let b = self.transform_point(p);
        dst.add_point(b);
        self.move_pen(p);
    }
    /// Add a line from pen to a point.
    ///
    /// * `bx` X-position of end point.
    /// * `by` Y-position of end point.
    fn line_to<D: PlotDest>(&mut self, dst: &mut D, bx: f32, by: f32) {
        let p = Vec2w::new(bx, by, self.s_width);
        let b = self.transform_point(p);
        dst.add_point(b);
        self.move_pen(p);
    }
    /// Add a quadratic bézier spline.
    ///
    /// The points are A (current pen position), B (control point), and C
    /// (spline end point).
    ///
    /// * `bx` X-position of control point.
    /// * `by` Y-position of control point.
    /// * `cx` X-position of end point.
    /// * `cy` Y-position of end point.
    fn quad_to<D: PlotDest>(&mut self, dst: &mut D, bx: f32, by: f32, cx: f32,
        cy: f32)
    {
        let pen = self.pen;
        let bb = Vec2w::new(bx, by, (pen.w + self.s_width) / 2.0);
        let cc = Vec2w::new(cx, cy, self.s_width);
        let a = self.transform_point(pen);
        let b = self.transform_point(bb);
        let c = self.transform_point(cc);
        self.quad_to_tran(dst, a, b, c);
        self.move_pen(cc);
    }
    /// Add a quadratic bézier spline.
    ///
    /// The spline is decomposed into a series of lines using the DeCastlejau
    /// method.
    fn quad_to_tran<D: PlotDest>(&mut self, dst: &mut D, a: Vec2w, b: Vec2w,
        c: Vec2w)
    {
        let ab    = a.midpoint(b);
        let bc    = b.midpoint(c);
        let ab_bc = ab.midpoint(bc);
        let ac    = a.midpoint(c);
        if self.is_within_tolerance(ab_bc, ac) {
            dst.add_point(c);
        } else {
            self.quad_to_tran(dst, a, ab, ab_bc);
            self.quad_to_tran(dst, ab_bc, bc, c);
        }
    }
    /// Check if two points are within tolerance threshold.
    fn is_within_tolerance(&self, a: Vec2w, b: Vec2w) -> bool {
        self.is_within_tolerance2(a.v, b.v)
    }
    /// Check if two points are within tolerance threshold.
    fn is_within_tolerance2(&self, a: Vec2, b: Vec2) -> bool {
        assert!(self.tol_sq > 0.0);
        a.dist_sq(b) <= self.tol_sq
    }
    /// Add a cubic bézier spline.
    ///
    /// The points are A (current pen position), B (first control point), C
    /// (second control point) and D (spline end point).
    ///
    /// * `bx` X-position of first control point.
    /// * `by` Y-position of first control point.
    /// * `cx` X-position of second control point.
    /// * `cy` Y-position of second control point.
    /// * `dx` X-position of end point.
    /// * `dy` Y-position of end point.
    fn cubic_to<D: PlotDest>(&mut self, dst: &mut D, bx: f32, by: f32,
        cx: f32, cy: f32, dx: f32, dy:f32)
    {
        let pen = self.pen;
        let bw = float_lerp(pen.w, self.s_width, 1.0 / 3.0);
        let cw = float_lerp(pen.w, self.s_width, 2.0 / 3.0);
        let bb = Vec2w::new(bx, by, bw);
        let cc = Vec2w::new(cx, cy, cw);
        let dd = Vec2w::new(dx, dy, self.s_width);
        let a = self.transform_point(pen);
        let b = self.transform_point(bb);
        let c = self.transform_point(cc);
        let d = self.transform_point(dd);
        self.cubic_to_tran(dst, a, b, c, d);
        self.move_pen(dd);
    }
    /// Add a cubic bézier spline.
    ///
    /// The spline is decomposed into a series of lines using the DeCastlejau
    /// method.
    fn cubic_to_tran<D: PlotDest>(&mut self, dst: &mut D, a: Vec2w, b: Vec2w,
        c: Vec2w, d: Vec2w)
    {
        let ab    = a.midpoint(b);
        let bc    = b.midpoint(c);
        let cd    = c.midpoint(d);
        let ab_bc = ab.midpoint(bc);
        let bc_cd = bc.midpoint(cd);
        let e     = ab_bc.midpoint(bc_cd);
        let ad    = a.midpoint(d);
        if self.is_within_tolerance(e, ad) {
            dst.add_point(d);
        } else {
            self.cubic_to_tran(dst, a, ab, ab_bc, e);
            self.cubic_to_tran(dst, e, bc_cd, cd, d);
        }
    }
    /// Fill path onto the mask.
    ///
    /// * `ops` PathOp iterator.
    /// * `rule` Fill rule.
    pub fn fill<'a, T>(&mut self, ops: T, rule: FillRule) -> &mut Raster<Mask8>
        where T: IntoIterator, T::Item: Borrow<PathOp>
    {
        let mut fig = Fig::new();
        self.add_ops(ops, &mut fig);
        // Closing figure required to handle coincident start/end points
        fig.close();
        fig.fill(&mut self.mask, &mut self.sgn_area[..], rule);
        &mut self.mask
    }
    /// Stroke path onto the mask.
    ///
    /// * `ops` PathOp iterator.
    pub fn stroke<'a, T>(&mut self, ops: T) -> &mut Raster<Mask8>
        where T: IntoIterator, T::Item: Borrow<PathOp>
    {
        let mut stroke = Stroke::new(self.join_style, self.tol_sq);
        self.add_ops(ops, &mut stroke);
        let ops = stroke.path_ops();
        self.fill(ops.iter(), FillRule::NonZero)
    }
    /// Get the mask.
    pub fn mask(&mut self) -> &mut Raster<Mask8> {
        &mut self.mask
    }
}
