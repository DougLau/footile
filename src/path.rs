// path.rs      2D vector paths.
//
// Copyright (c) 2017  Douglas P Lau
//
use std::slice::Iter;

/// Fill-rule for filling paths.
#[derive(Clone,Copy,Debug)]
pub enum FillRule {
    /// All points within bounds are filled
    NonZero,
    /// Alternate filling with path outline
    EvenOdd,
}

/// Style for joins.
#[derive(Clone,Copy,Debug)]
pub enum JoinStyle {
    /// Mitered join with limit (miter length to stroke width ratio)
    Miter(f32),
    /// Beveled join
    Bevel,
    /// Rounded join
    Round,
}

/// Path operation.
pub enum PathOp {
    Close(),
    Move(f32, f32),
    Line(f32, f32),
    Quad(f32, f32, f32, f32),
    Cubic(f32, f32, f32, f32, f32, f32),
    PenWidth(f32),
}

/// Path2D is a container for arbitrary path operations.
///
/// Use [PathBuilder](struct.PathBuilder.html) to construct paths.
pub struct Path2D {
    ops : Vec<PathOp>,
}

/// Builder for [Path2D](struct.Path2D.html).
///
/// # Example
/// ```
/// use footile::PathBuilder;
/// let path = PathBuilder::new()
///                        .move_to(10f32, 10f32)
///                        .line_to(90f32, 90f32)
///                        .build();
/// ```
pub struct PathBuilder {
    ops      : Vec<PathOp>,
    absolute : bool,
    pen_x    : f32,
    pen_y    : f32,
}

impl Path2D {
    /// Get an iterator of path operations.
    pub fn iter(&self) -> Iter<PathOp> {
        self.ops.iter()
    }
}

impl PathBuilder {
    /// Create a new PathBuilder.
    pub fn new() -> PathBuilder {
        let ops = Vec::with_capacity(32);
        PathBuilder {
            ops,
            absolute : false,
            pen_x    : 0f32,
            pen_y    : 0f32,
        }
    }
    /// Use absolute coordinates for subsequent operations.
    ///
    /// This is the default setting.
    pub fn absolute(mut self) -> Self {
        self.absolute = true;
        self
    }
    /// Use relative coordinates for subsequent operations.
    pub fn relative(mut self) -> Self {
        self.absolute = false;
        self
    }
    /// Get absolute point.
    fn pt(&self, x: f32, y: f32) -> (f32, f32) {
        if self.absolute {
            (x, y)
        } else {
            (self.pen_x + x, self.pen_y + y)
        }
    }
    /// Close current sub-path and move pen to origin.
    pub fn close(mut self) -> Self {
        self.ops.push(PathOp::Close());
        self.pen_x = 0f32;
        self.pen_y = 0f32;
        self
    }
    /// Move pen to a point.
    ///
    /// * `bx` X-position of point.
    /// * `by` Y-position of point.
    pub fn move_to(mut self, bx: f32, by: f32) -> Self {
        let (abx, aby) = self.pt(bx, by);
        self.ops.push(PathOp::Move(abx, aby));
        self.pen_x = abx;
        self.pen_y = aby;
        self
    }
    /// Add a line from pen to a point.
    ///
    /// * `bx` X-position of end point.
    /// * `by` Y-position of end point.
    pub fn line_to(mut self, bx: f32, by: f32) -> Self {
        let (abx, aby) = self.pt(bx, by);
        self.ops.push(PathOp::Line(abx, aby));
        self.pen_x = abx;
        self.pen_y = aby;
        self
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
    pub fn quad_to(mut self, bx: f32, by: f32, cx: f32, cy: f32) -> Self {
        let (abx, aby) = self.pt(bx, by);
        let (acx, acy) = self.pt(cx, cy);
        self.ops.push(PathOp::Quad(abx, aby, acx, acy));
        self.pen_x = acx;
        self.pen_y = acy;
        self
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
    pub fn cubic_to(mut self, bx: f32, by: f32, cx: f32, cy: f32, dx: f32,
                    dy: f32) -> Self
    {
        let (abx, aby) = self.pt(bx, by);
        let (acx, acy) = self.pt(cx, cy);
        let (adx, ady) = self.pt(dx, dy);
        self.ops.push(PathOp::Cubic(abx, aby, acx, acy, adx, ady));
        self.pen_x = adx;
        self.pen_y = ady;
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
    /// Build path from specified operations.
    pub fn build(self) -> Path2D {
        Path2D {
            ops : self.ops,
        }
    }
}
