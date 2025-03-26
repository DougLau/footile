// path.rs      2D vector paths.
//
// Copyright (c) 2017-2025  Douglas P Lau
//
use pointy::Pt;

/// Fill-rule for filling paths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FillRule {
    /// All points within bounds are filled
    NonZero,
    /// Alternate filling with path outline
    EvenOdd,
}

/// Path operation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PathOp {
    /// Close the path
    Close(),
    /// Move to a point
    Move(Pt<f32>),
    /// Straight line to end point
    Line(Pt<f32>),
    /// Quadratic bézier curve (control point and end point)
    Quad(Pt<f32>, Pt<f32>),
    /// Cubic bézier curve (two control points and end point)
    Cubic(Pt<f32>, Pt<f32>, Pt<f32>),
    /// Set pen width (for stroking)
    PenWidth(f32),
    /// Set transformation
    Transform(TransformOp),
}

/// Path transform operation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TransformOp {
    /// Translate is user points
    Translate(f32, f32),
    /// Scale
    Scale(f32, f32),
    /// Rotate in radians
    Rotate(f32),
    /// Skew radians
    Skew(f32, f32),
    /// Reset transformations
    None
}

/// A `Path2D` is a builder for `Vec<PathOp>`.
///
/// # Example
/// ```
/// use footile::Path2D;
///
/// let path = Path2D::default()
///     .move_to(10.0, 10.0)
///     .line_to(90.0, 90.0)
///     .finish();
/// ```
pub struct Path2D {
    /// Vec of path operations
    ops: Vec<PathOp>,
    /// Absolute vs relative coordinates
    absolute: bool,
    /// Current pen position
    pen: Pt<f32>,
}

impl Default for Path2D {
    fn default() -> Path2D {
        let ops = Vec::with_capacity(32);
        Path2D {
            ops,
            absolute: false,
            pen: Pt::default(),
        }
    }
}

impl Path2D {
    /// Use absolute coordinates for subsequent operations.
    pub fn absolute(mut self) -> Self {
        self.absolute = true;
        self
    }

    /// Use relative coordinates for subsequent operations.
    ///
    /// This is the default setting.
    pub fn relative(mut self) -> Self {
        self.absolute = false;
        self
    }

    /// Get absolute point.
    fn pt(&self, x: f32, y: f32) -> Pt<f32> {
        if self.absolute {
            Pt::new(x, y)
        } else {
            Pt::new(self.pen.x + x, self.pen.y + y)
        }
    }

    /// Close current sub-path and move pen to origin.
    pub fn close(mut self) -> Self {
        self.ops.push(PathOp::Close());
        self.pen = Pt::default();
        self
    }

    /// Move the pen to a point.
    ///
    /// * `x` X-position of point.
    /// * `y` Y-position of point.
    pub fn move_to(mut self, x: f32, y: f32) -> Self {
        let pb = self.pt(x, y);
        self.ops.push(PathOp::Move(pb));
        self.pen = pb;
        self
    }

    /// Add a line from pen to a point.
    ///
    /// * `x` X-position of end point.
    /// * `y` Y-position of end point.
    pub fn line_to(mut self, x: f32, y: f32) -> Self {
        let pb = self.pt(x, y);
        self.ops.push(PathOp::Line(pb));
        self.pen = pb;
        self
    }

    /// Add a quadratic bézier spline.
    ///
    /// The points are:
    ///
    /// * Current pen position: P<sub>a</sub>
    /// * Control point: P<sub>b</sub> (`bx` / `by`)
    /// * Spline end point: P<sub>c</sub> (`cx` / `cy`)
    pub fn quad_to(mut self, bx: f32, by: f32, cx: f32, cy: f32) -> Self {
        let pb = self.pt(bx, by);
        let pc = self.pt(cx, cy);
        self.ops.push(PathOp::Quad(pb, pc));
        self.pen = pc;
        self
    }

    /// Add a cubic bézier spline.
    ///
    /// The points are:
    ///
    /// * Current pen position: P<sub>a</sub>
    /// * First control point: P<sub>b</sub> (`bx` / `by`)
    /// * Second control point: P<sub>c</sub> (`cx` / `cy`)
    /// * Spline end point: P<sub>d</sub> (`dx` / `dy`)
    pub fn cubic_to(
        mut self,
        bx: f32,
        by: f32,
        cx: f32,
        cy: f32,
        dx: f32,
        dy: f32,
    ) -> Self {
        let pb = self.pt(bx, by);
        let pc = self.pt(cx, cy);
        let pd = self.pt(dx, dy);
        self.ops.push(PathOp::Cubic(pb, pc, pd));
        self.pen = pd;
        self
    }

    /// Set pen stroke width.
    ///
    /// All subsequent path points will be affected, until the stroke width
    /// is changed again.
    ///
    /// * `width` Pen stroke width.
    pub fn pen_width(mut self, width: f32) -> Self {
        self.ops.push(PathOp::PenWidth(width));
        self
    }

    /// Set transformation
    /// 
    /// All subsequent path points will be affected, until the transform
    /// is changed again.
    /// * `transform` Optional transformation
    pub fn transform(mut self, transform: TransformOp) -> Self {
        self.ops.push(PathOp::Transform(transform));
        self
    }
    


    /// Finish path with specified operations.
    pub fn finish(self) -> Vec<PathOp> {
        self.ops
    }
}
