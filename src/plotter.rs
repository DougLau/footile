// plotter.rs      Vector path plotter.
//
// Copyright (c) 2017-2025  Douglas P Lau
//
use crate::fig::Fig;
use crate::geom::{float_lerp, WidePt};
use crate::path::{FillRule, PathOp};
use crate::ink::{ColorInk, Ink};
use crate::stroker::{JoinStyle, Stroke};
use pix::chan::{Ch8, Linear, Premultiplied};
use pix::el::Pixel;
use pix::Raster;
use pointy::{Pt, Transform};
use std::borrow::Borrow;

/// Plotter for 2D vector [path]s.
///
/// This is a software vector rasterizer featuring anti-aliasing.  The plotter
/// contains a raster, which is drawn by fill and stroke calls.
///
/// [path]: struct.Path2D.html
///
/// # Example
/// ```
/// use footile::{Path2D, Plotter};
/// use pix::rgb::Rgba8p;
/// use pix::Raster;
///
/// let path = Path2D::default()
///     .pen_width(3.0)
///     .move_to(50.0, 34.0)
///     .cubic_to(4.0, 16.0, 16.0, 28.0, 0.0, 32.0)
///     .cubic_to(-16.0, -4.0, -4.0, -16.0, 0.0, -32.0)
///     .close()
///     .finish();
/// let mut p = Plotter::new(Raster::with_clear(100, 100));
/// p.stroke(&path, Rgba8p::new(255, 128, 0, 255));
/// ```
pub struct Plotter<P>
where
    P: Pixel<Chan = Ch8, Alpha = Premultiplied, Gamma = Linear>,
{
    /// Image raster
    raster: Raster<P>,
    /// Signed area buffer
    sgn_area: Vec<i16>,
    /// Current pen position and width
    pen: WidePt,
    /// User to pixel affine transform
    transform: Transform<f32>,
    /// Curve decomposition tolerance squared
    tol_sq: f32,
    /// Current stroke width
    s_width: f32,
    /// Current join style
    join_style: JoinStyle,
}

/// Plot destination
trait PlotDest {
    /// Add a point.
    ///
    /// * `pt` Point to add (w indicates stroke width).
    fn add_point(&mut self, pt: WidePt);

    /// Close the current sub-figure.
    ///
    /// * `joined` If true, join ends of sub-plot.
    fn close(&mut self, joined: bool);
}

impl PlotDest for Fig {
    fn add_point(&mut self, pt: WidePt) {
        Fig::add_point(self, pt.0);
    }
    fn close(&mut self, _joined: bool) {
        Fig::close(self);
    }
}

impl PlotDest for Stroke {
    fn add_point(&mut self, pt: WidePt) {
        Stroke::add_point(self, pt);
    }
    fn close(&mut self, joined: bool) {
        Stroke::close(self, joined);
    }
}

impl<P> Plotter<P>
where
    P: Pixel<Chan = Ch8, Alpha = Premultiplied, Gamma = Linear>,
{
    /// Create a new plotter.
    ///
    /// * `raster` Raster to draw.
    pub fn new(raster: Raster<P>) -> Self {
        let tol = 0.3;
        let len = raster.width() as usize;
        // Capacity must be 8-element multiple (for SIMD)
        let cap = ((len + 7) >> 3) << 3;
        let mut sgn_area = vec![0; cap];
        // Remove excess elements
        for _ in 0..cap - len {
            sgn_area.pop();
        }
        Plotter {
            raster,
            sgn_area,
            pen: WidePt::default(),
            transform: Transform::default(),
            tol_sq: tol * tol,
            s_width: 1.0,
            join_style: JoinStyle::Miter(4.0),
        }
    }

    /// Get width in pixels.
    pub fn width(&self) -> u32 {
        self.raster.width()
    }

    /// Get height in pixels.
    pub fn height(&self) -> u32 {
        self.raster.height()
    }

    /// Reset pen.
    fn reset(&mut self) {
        self.pen = WidePt(Pt::default(), self.s_width);
    }

    /// Set tolerance threshold for curve decomposition.
    pub fn set_tolerance(&mut self, t: f32) -> &mut Self {
        let tol = t.max(0.01);
        self.tol_sq = tol * tol;
        self
    }

    /// Set the transform.
    pub fn set_transform(&mut self, t: Transform<f32>) -> &mut Self {
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
    fn move_pen(&mut self, p: WidePt) {
        self.pen = p;
    }

    /// Transform a point.
    fn transform_point(&self, p: WidePt) -> WidePt {
        let pt = self.transform * p.0;
        WidePt(pt, p.w())
    }

    /// Add a series of ops.
    fn add_ops<T, D>(&mut self, ops: T, dst: &mut D)
    where
        T: IntoIterator,
        T::Item: Borrow<PathOp>,
        D: PlotDest,
    {
        self.reset();
        for op in ops {
            self.add_op(dst, op.borrow());
        }
    }

    /// Add a path operation.
    fn add_op<D: PlotDest>(&mut self, dst: &mut D, op: &PathOp) {
        match *op {
            PathOp::Close() => self.close(dst),
            PathOp::Move(pb) => self.move_to(dst, pb),
            PathOp::Line(pb) => self.line_to(dst, pb),
            PathOp::Quad(pb, pc) => self.quad_to(dst, pb, pc),
            PathOp::Cubic(pb, pc, pd) => self.cubic_to(dst, pb, pc, pd),
            PathOp::PenWidth(w) => self.pen_width(w),
        };
    }

    /// Close current sub-path and move pen to origin.
    fn close<D: PlotDest>(&mut self, dst: &mut D) {
        dst.close(true);
        self.reset();
    }

    /// Move pen to a point.
    ///
    /// * `pb` New point.
    fn move_to<D: PlotDest>(&mut self, dst: &mut D, pb: Pt<f32>) {
        let p = WidePt(pb, self.s_width);
        dst.close(false);
        let b = self.transform_point(p);
        dst.add_point(b);
        self.move_pen(p);
    }

    /// Add a line from pen to a point.
    ///
    /// * `pb` End point.
    fn line_to<D: PlotDest>(&mut self, dst: &mut D, pb: Pt<f32>) {
        let p = WidePt(pb, self.s_width);
        let b = self.transform_point(p);
        dst.add_point(b);
        self.move_pen(p);
    }

    /// Add a quadratic bézier spline.
    ///
    /// The points are A (current pen position), B (control point), and C
    /// (spline end point).
    ///
    /// * `cp` Control point.
    /// * `end` End point.
    fn quad_to<D: PlotDest>(&mut self, dst: &mut D, cp: Pt<f32>, end: Pt<f32>) {
        let pen = self.pen;
        let bb = WidePt(cp, (pen.w() + self.s_width) / 2.0);
        let cc = WidePt(end, self.s_width);
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
    fn quad_to_tran<D: PlotDest>(
        &self,
        dst: &mut D,
        a: WidePt,
        b: WidePt,
        c: WidePt,
    ) {
        let ab = a.midpoint(b);
        let bc = b.midpoint(c);
        let ab_bc = ab.midpoint(bc);
        let ac = a.midpoint(c);
        if self.is_within_tolerance(ab_bc, ac) {
            dst.add_point(c);
        } else {
            self.quad_to_tran(dst, a, ab, ab_bc);
            self.quad_to_tran(dst, ab_bc, bc, c);
        }
    }

    /// Check if two points are within tolerance threshold.
    fn is_within_tolerance(&self, a: WidePt, b: WidePt) -> bool {
        self.is_within_tolerance2(a.0, b.0)
    }

    /// Check if two points are within tolerance threshold.
    fn is_within_tolerance2(&self, a: Pt<f32>, b: Pt<f32>) -> bool {
        assert!(self.tol_sq > 0.0);
        a.distance_sq(b) <= self.tol_sq
    }

    /// Add a cubic bézier spline.
    ///
    /// The points are A (current pen position), B (first control point), C
    /// (second control point) and D (spline end point).
    ///
    /// * `cp0` First control point.
    /// * `cp1` Second control point.
    /// * `end` End point.
    fn cubic_to<D: PlotDest>(
        &mut self,
        dst: &mut D,
        cp0: Pt<f32>,
        cp1: Pt<f32>,
        end: Pt<f32>,
    ) {
        let pen = self.pen;
        let w0 = float_lerp(pen.w(), self.s_width, 1.0 / 3.0);
        let w1 = float_lerp(pen.w(), self.s_width, 2.0 / 3.0);
        let bb = WidePt(cp0, w0);
        let cc = WidePt(cp1, w1);
        let dd = WidePt(end, self.s_width);
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
    fn cubic_to_tran<D: PlotDest>(
        &self,
        dst: &mut D,
        pa: WidePt,
        pb: WidePt,
        pc: WidePt,
        pd: WidePt,
    ) {
        let ab = pa.midpoint(pb);
        let bc = pb.midpoint(pc);
        let cd = pc.midpoint(pd);
        let ab_bc = ab.midpoint(bc);
        let bc_cd = bc.midpoint(cd);
        let pe = ab_bc.midpoint(bc_cd);
        let ad = pa.midpoint(pd);
        if self.is_within_tolerance(pe, ad) {
            dst.add_point(pd);
        } else {
            self.cubic_to_tran(dst, pa, ab, ab_bc, pe);
            self.cubic_to_tran(dst, pe, bc_cd, cd, pd);
        }
    }

    /// Fill path onto the raster.
    ///
    /// * `rule` Fill rule.
    /// * `ops` PathOp iterator.
    /// * `clr` Color to fill.
    pub fn fill<T>(&mut self, rule: FillRule, ops: T, clr: P) -> &mut Raster<P>
    where
        T: IntoIterator,
        T::Item: Borrow<PathOp>,
    {
        self.fill_with(rule, ops, ColorInk { clr })
    }

    /// Fill the figure to an image raster.
    ///
    /// * `rule` Fill rule.
    /// * `ops` PathOp iterator.
    /// * `ink` Determines how to fill row pixels.
    pub fn fill_with<T, R>(
        &mut self,
        rule: FillRule,
        ops: T,
        ink: R,
    ) -> &mut Raster<P>
    where
        R: Ink<P>,
        T: IntoIterator,
        T::Item: Borrow<PathOp>,
    {
        let mut fig = Fig::new();
        self.add_ops(ops, &mut fig);
        // Closing figure required to handle coincident start/end points
        fig.close();
        fig.fill_with(rule, &mut self.raster, ink, &mut self.sgn_area[..]);
        &mut self.raster
    }

    /// Stroke path onto the raster.
    ///
    /// * `ops` PathOp iterator.
    /// * `clr` Color to stroke.
    pub fn stroke<T>(&mut self, ops: T, clr: P) -> &mut Raster<P>
    where
        T: IntoIterator,
        T::Item: Borrow<PathOp>,
    {
        self.stroke_with(ops, ColorInk { clr })
    }

    /// Stroke path onto the raster.
    ///
    /// * `ops` PathOp iterator.
    /// * `clr` Color to stroke.
    pub fn stroke_with<T, R>(&mut self, ops: T, ink: R) -> &mut Raster<P>
    where
        T: IntoIterator,
        T::Item: Borrow<PathOp>,
        R: Ink<P>,
    {
        let mut stroke = Stroke::new(self.join_style, self.tol_sq);
        self.add_ops(ops, &mut stroke);
        let ops = stroke.path_ops();
        self.fill_with(FillRule::NonZero, ops.iter(), ink)
    }

    /// Get a reference to the raster.
    pub fn raster(&self) -> &Raster<P> {
        &self.raster
    }

    /// Get a mutable reference to the raster.
    pub fn raster_mut(&mut self) -> &mut Raster<P> {
        &mut self.raster
    }

    /// Consume the plotter and get the raster.
    pub fn into_raster(self) -> Raster<P> {
        self.raster
    }
}

#[cfg(test)]
mod test {
    use crate::*;
    use pix::matte::Matte8;
    use pix::Raster;

    #[test]
    fn overlapping() {
        let path = Path2D::default()
            .absolute()
            .move_to(8.0, 4.0)
            .line_to(8.0, 3.0)
            .cubic_to(8.0, 3.0, 8.0, 3.0, 9.0, 3.75)
            .line_to(8.0, 3.75)
            .line_to(8.5, 3.75)
            .line_to(8.5, 3.5)
            .finish();
        let r = Raster::with_clear(16, 16);
        let mut p = Plotter::new(r);
        p.fill(FillRule::NonZero, &path, Matte8::new(255));
    }
}
