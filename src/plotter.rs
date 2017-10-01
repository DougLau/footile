/// plotter.rs      Vector path plotter.
///
/// Copyright (c) 2017  Douglas P Lau
///
/// A plotter is a simple vector path plotter.  Paths are drawn
/// using lines and splines (quadratic or cubic).
///
use super::fig::{ Fig, FillRule };
use super::geom::{ Vec2, Vec3, float_lerp };
use super::mask::Mask;

/// Plotter struct
pub struct Plotter {
    fig       : Fig,    // drawing fig
    sfig      : Fig,    // stroking fig
    mask      : Mask,   // image mask
    scan_buf  : Mask,   // scan line buffer
    pen       : Vec3,   // current pen position
    s_width   : f32,    // current stroke width
    tol_sq    : f32,    // spline tolerance squared
    scale     : f32,    // scale factor
}

impl Plotter {
    /// Create a plotter
    pub fn new(width: u32, height: u32, tol: f32) -> Plotter {
        Plotter {
            fig: Fig::new(),
            sfig: Fig::new(),
            mask: Mask::new(width, height),
            scan_buf: Mask::new(width, 1),
            pen: Vec3::new(0f32, 0f32, 1f32),
            s_width: 1f32,
            tol_sq: tol * tol,
            scale: 1f32,
        }
    }
    /// Get the width
    pub fn width(&self) -> u32 {
        self.mask.width()
    }
    /// Get the height
    pub fn height(&self) -> u32 {
        self.mask.height()
    }
    /// Set the plot data size
    pub fn data_size(&mut self, width: u32, height: u32) {
        let sx = self.width() as f32 / width as f32;
        let sy = self.height() as f32 / height as f32;
        self.scale = sx.min(sy);
        self.s_width = self.scale;
    }
    /// Reset the plotter
    pub fn reset(&mut self) {
        self.fig.reset();
        self.mask.reset();
    }
    /// Add a line
    pub fn line_to(&mut self, b: Vec2) {
        let x = self.pen.x + b.x * self.scale;
        let y = self.pen.y + b.y * self.scale;
        let w = self.s_width;
        self.line_to_scaled(Vec3::new(x, y, w));
    }
    /// Add a line
    fn line_to_scaled(&mut self, p: Vec3) {
        self.pen = p;
        self.fig.add_point(p);
    }
    /// Add a quadratic bezier spline
    ///
    /// The points are A, B and C.  A is the current pen position.  B is the
    /// control point.  C is the final pen position.
    pub fn quad_to(&mut self, b: Vec2, c: Vec2) {
        let bb = Vec3::new(
            self.pen.x + b.x * self.scale,
            self.pen.y + b.y * self.scale,
            (self.pen.z + self.s_width) / 2f32,
        );
        let cc = Vec3::new(
            self.pen.x + c.x * self.scale,
            self.pen.y + c.y * self.scale,
            self.s_width,
        );
        self.quad_to_scaled(bb, cc);
    }
    /// Add a quadratic bezier spline
    ///
    /// The spline is decomposed into a series of lines using the DeCastlejau
    /// method.  The points are A, B and C.  A is the current pen position.  B
    /// is the control point.  C is the final pen position.
    fn quad_to_scaled(&mut self, b: Vec3, c: Vec3) {
        let a     = self.pen;
        let ab    = a.midpoint(b);
        let bc    = b.midpoint(c);
        let ab_bc = ab.midpoint(bc);
        let ac    = a.midpoint(c);
        if self.is_within_tolerance(ab_bc, ac) {
            self.line_to_scaled(c);
        } else {
            self.quad_to_scaled(ab, ab_bc);
            self.quad_to_scaled(bc, c);
        }
    }
    /// Check if two points are within the tolerance threshold
    fn is_within_tolerance(&self, a: Vec3, b: Vec3) -> bool {
        let a2 = Vec2::new(a.x, a.y);
        let b2 = Vec2::new(b.x, b.y);
        a2.dist_sq(b2) <= self.tol_sq
    }
    /// Add a cubic bezier spline
    ///
    /// The points are A, B, C and D.  A is the current pen position.  B and C
    /// are the two control points.  D is the final pen position.
    pub fn cubic_to(&mut self, b: Vec2, c: Vec2, d: Vec2) {
        let bb = Vec3::new(
            self.pen.x + b.x * self.scale,
            self.pen.y + b.y * self.scale,
            float_lerp(self.pen.z, self.s_width, 1f32 / 3f32),
        );
        let cc = Vec3::new(
            self.pen.x + c.x * self.scale,
            self.pen.y + c.y * self.scale,
            float_lerp(self.pen.z, self.s_width, 2f32 / 3f32),
        );
        let dd = Vec3::new(
            self.pen.x + d.x * self.scale,
            self.pen.y + d.y * self.scale,
            self.s_width,
        );
        self.cubic_to_scaled(bb, cc, dd);
    }
    /// Add a cubic bezier spline
    ///
    /// The spline is decomposed into a series of lines using the DeCastlejau
    /// method.  The points are A, B, C and D.  A is the current pen position.
    /// B and C are the two control points.  D is the final pen position.
    fn cubic_to_scaled(&mut self, b: Vec3, c: Vec3, d: Vec3) {
        let a     = self.pen;
        let ab    = a.midpoint(b);
        let bc    = b.midpoint(c);
        let cd    = c.midpoint(d);
        let ab_bc = ab.midpoint(bc);
        let bc_cd = bc.midpoint(cd);
        let e     = ab_bc.midpoint(bc_cd);
        let ad    = a.midpoint(d);
        if self.is_within_tolerance(e, ad) {
            self.line_to_scaled(d);
        } else {
            self.cubic_to_scaled(ab, ab_bc, e);
            self.cubic_to_scaled(bc_cd, cd, d);
        }
    }
    /// Close the current sub-fig
    pub fn close(&mut self, joined: bool) {
        self.fig.close(joined);
        self.pen.x = 0f32;
        self.pen.y = 0f32;
    }
    /// Set the pen stroking width
    pub fn pen_width(&mut self, width: f32, pixels: bool) {
        self.s_width = if pixels { width } else { width * self.scale }
    }
    /// Rasterize (fill) onto the mask
    pub fn rasterize_fill(&mut self, rule: FillRule, reset: bool) {
        self.fig.fill(&mut self.mask, &mut self.scan_buf, rule);
        if reset {
            self.pen.x = 0f32;
            self.pen.y = 0f32;
            self.fig.reset();
        }
    }
    /// Rasterize (stroke) onto the mask
    pub fn rasterize_stroke(&mut self, reset: bool) {
        self.fig.stroke(&mut self.sfig);
        self.sfig.fill(&mut self.mask, &mut self.scan_buf, FillRule::NonZero);
        if reset {
            self.pen.x = 0f32;
            self.pen.y = 0f32;
            self.fig.reset();
            self.sfig.reset();
        }
    }
    /// Get a mutable reference to the mask
    pub fn get_mask(&mut self) -> &mut Mask {
        &mut self.mask
    }
}
