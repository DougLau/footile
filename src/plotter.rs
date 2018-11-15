// plotter.rs      Vector path plotter.
//
// Copyright (c) 2017-2018  Douglas P Lau
//
use std::io;
use fig::Fig;
use geom::{Transform, Vec2, Vec2w, float_lerp};
use path::{FillRule, JoinStyle, PathOp};
use mask::Mask;
use raster::{Color,Raster};
use stroker::Stroke;

/// Plotter for 2D vector paths.
///
/// This is a software vector rasterizer featuring high quality anti-aliasing.
/// Paths can be created using [PathBuilder](struct.PathBuilder.html).
/// The plotter contains a [Mask](struct.Mask.html) of the current plot, which
/// is affected by fill and stroke calls.  Using the color_over method will
/// cause a [Raster](struct.Raster.html) to be created with the same height and
/// width as the mask.
///
/// # Example
/// ```
/// use footile::{PathBuilder, Plotter};
/// let path = PathBuilder::new().pen_width(3f32)
///                        .move_to(50f32, 34f32)
///                        .cubic_to(4f32, 16f32, 16f32, 28f32, 0f32, 32f32)
///                        .cubic_to(-16f32, -4f32, -4f32, -16f32, 0f32, -32f32)
///                        .close().build();
/// let mut p = Plotter::new(100, 100);
/// p.stroke(&path);
/// ```
pub struct Plotter {
    mask       : Mask,          // image mask
    raster     : Option<Raster>,// image raster
    sgn_area   : Vec<i16>,      // signed area buffer
    pen        : Vec2w,         // current pen position and width
    transform  : Transform,     // user to pixel affine transform
    tol_sq     : f32,           // curve decomposition tolerance squared
    s_width    : f32,           // current stroke width
    join_style : JoinStyle,     // current join style
}

/// Plot destination
trait PlotDest {
    /// Add a point.
    ///
    /// * `pt` Point to add (z indicates stroke width).
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
        let tol = 0.3f32;
        let w = if width > 0 { width } else { 100 };
        let h = if height > 0 { height } else { 100 };
        let len = w as usize;
        // Capacity must be 8-element multiple (for SIMD)
        let cap = ((len + 7) >> 3) << 3;
        let mut sgn_area = vec![0i16; cap];
        // Remove excess elements
        for _ in 0..cap-len { sgn_area.pop(); };
        Plotter {
            mask       : Mask::new(w, h),
            raster     : None,
            sgn_area   : sgn_area,
            pen        : Vec2w::new(0f32, 0f32, 1f32),
            transform  : Transform::new(),
            tol_sq     : tol * tol,
            s_width    : 1f32,
            join_style : JoinStyle::Miter(4f32),
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
        self.pen = Vec2w::new(0f32, 0f32, self.s_width);
    }
    /// Clear the mask.
    pub fn clear_mask(&mut self) -> &mut Self {
        self.mask.clear();
        self
    }
    /// Set tolerance threshold for curve decomposition.
    pub fn set_tolerance(&mut self, t: f32) -> &mut Self {
        let tol = t.max(0.01f32);
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
        where T: IntoIterator<Item=&'a PathOp>, D: PlotDest
    {
        self.reset();
        for op in ops {
            self.add_op(dst, op);
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
        let bb = Vec2w::new(bx, by, (pen.w + self.s_width) / 2f32);
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
        assert!(self.tol_sq > 0f32);
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
        let bw = float_lerp(pen.w, self.s_width, 1f32 / 3f32);
        let cw = float_lerp(pen.w, self.s_width, 2f32 / 3f32);
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
    pub fn fill<'a, T>(&mut self, ops: T, rule: FillRule) -> &mut Self
        where T: IntoIterator<Item=&'a PathOp>
    {
        let mut fig = Fig::new();
        self.add_ops(ops, &mut fig);
        // Closing figure required to handle coincident start/end points
        fig.close();
        fig.fill(&mut self.mask, &mut self.sgn_area[..], rule);
        self
    }
    /// Stroke path onto the mask.
    ///
    /// * `ops` PathOp iterator.
    pub fn stroke<'a, T>(&mut self, ops: T) -> &mut Self
        where T: IntoIterator<Item=&'a PathOp>
    {
        let mut stroke = Stroke::new(self.join_style, self.tol_sq);
        self.add_ops(ops, &mut stroke);
        let ops = stroke.path_ops();
        self.fill(ops.iter(), FillRule::NonZero)
    }
    /// Composite mask with a color onto raster, using "over".
    ///
    /// * `clr` Color to composite.
    pub fn color_over(&mut self, clr: Color) -> &mut Self {
        if self.raster.is_none() {
            self.raster = Some(Raster::new(self.width(), self.height()));
        }
        if let Some(mut r) = self.raster.take() {
            r.color_over(self.mask(), clr);
            self.raster = Some(r);
        }
        self.clear_mask()
    }
    /// Get the mask.
    pub fn mask(&self) -> &Mask {
        &self.mask
    }
    /// Get the raster.
    pub fn raster(&self) -> Option<&Raster> {
        self.raster.as_ref()
    }
    /// Write the plot to a PNG (portable network graphics) file.
    ///
    /// * `filename` Name of file to write.
    pub fn write_png(&mut self, filename: &str) -> io::Result<()> {
        if let Some(r) = self.raster.take() {
            r.write_png(filename)
        } else {
            self.mask.write_png(filename)
        }
    }
}
