// stroker.rs   A path stroker.
//
// Copyright (c) 2017-2021  Douglas P Lau
//
use crate::geom::WidePt;
use crate::path::PathOp;
use crate::vid::Vid;
use pointy::{Line, Pt};
use std::fmt;

/// Style for stroke joins.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum JoinStyle {
    /// Mitered join with limit (miter length to stroke width ratio)
    Miter(f32),
    /// Beveled join
    Bevel,
    /// Rounded join
    Round,
}

/// Stroke direction enum
#[derive(Clone, Copy, Debug, PartialEq)]
enum Dir {
    Forward,
    Reverse,
}

/// Sub-stroke struct
#[derive(Clone)]
struct SubStroke {
    /// Starting point
    start: Vid,
    /// Number of points
    n_points: Vid,
    /// Joined ends flag
    joined: bool,
    /// Done flag
    done: bool,
}

/// Stroke struct
#[derive(Clone)]
pub struct Stroke {
    /// Join style
    join_style: JoinStyle,
    /// Tolerance squared
    tol_sq: f32,
    /// All points
    points: Vec<WidePt>,
    /// All sub-strokes
    subs: Vec<SubStroke>,
}

impl SubStroke {
    /// Create a new sub-stroke
    fn new(start: Vid) -> SubStroke {
        SubStroke {
            start,
            n_points: Vid(0),
            joined: false,
            done: false,
        }
    }

    /// Get next vertex within a sub-stroke
    fn next(&self, vid: Vid, dir: Dir) -> Vid {
        match dir {
            Dir::Forward => {
                let v = vid + 1;
                if v < self.start + self.n_points {
                    v
                } else {
                    self.start
                }
            }
            Dir::Reverse => {
                if vid > self.start {
                    vid - 1
                } else {
                    self.start + self.n_points - 1
                }
            }
        }
    }

    /// Get count of points
    fn len(&self) -> Vid {
        if self.joined {
            self.n_points + 1
        } else if self.n_points > Vid(0) {
            self.n_points - 1
        } else {
            Vid(0)
        }
    }
}

impl fmt::Debug for Stroke {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for sub in &self.subs {
            write!(f, "sub {:?}+{:?} ", sub.start, sub.n_points)?;
            let end = sub.start + sub.n_points;
            for v in usize::from(sub.start)..usize::from(end) {
                write!(f, "{:?} ", self.point(Vid::from(v)))?;
            }
        }
        Ok(())
    }
}

impl Stroke {
    /// Create a new stroke.
    pub fn new(join_style: JoinStyle, tol_sq: f32) -> Stroke {
        let points = Vec::with_capacity(1024);
        let mut subs = Vec::with_capacity(16);
        subs.push(SubStroke::new(Vid(0)));
        Stroke {
            join_style,
            tol_sq,
            points,
            subs,
        }
    }

    /// Check if two points are within tolerance threshold.
    fn is_within_tolerance2(&self, a: Pt<f32>, b: Pt<f32>) -> bool {
        assert!(self.tol_sq > 0.0);
        a.dist_sq(b) <= self.tol_sq
    }

    /// Get the count of sub-strokes
    fn len(&self) -> usize {
        self.subs.len()
    }

    /// Get start of a sub-strokes
    fn sub_start(&self, i: usize) -> Vid {
        self.subs[i].start
    }

    /// Get end of a sub-strokes
    fn sub_end(&self, i: usize) -> Vid {
        let sub = &self.subs[i];
        sub.next(sub.start, Dir::Reverse)
    }

    /// Check if a sub-stroke is joined
    fn sub_joined(&self, i: usize) -> bool {
        self.subs[i].joined
    }

    /// Get the number of points in a sub-stroke
    fn sub_points(&self, i: usize) -> Vid {
        self.subs[i].len()
    }

    /// Get the current sub-stroke
    fn sub_current(&mut self) -> &mut SubStroke {
        self.subs.last_mut().unwrap()
    }

    /// Add a new sub-stroke
    fn sub_add(&mut self) {
        let vid = Vid::from(self.points.len());
        self.subs.push(SubStroke::new(vid));
    }

    /// Add a point to the current sub-stroke
    fn sub_add_point(&mut self) {
        let sub = self.sub_current();
        sub.n_points += 1;
    }

    /// Get the sub-stroke at a specified vertex ID
    fn sub_at(&self, vid: Vid) -> &SubStroke {
        let n_subs = self.subs.len();
        for i in 0..n_subs {
            let sub = &self.subs[i];
            if vid < sub.start + sub.n_points {
                return sub;
            }
        }
        // Invalid vid indicates bug
        unreachable!();
    }

    /// Get next vertex
    fn next(&self, vid: Vid, dir: Dir) -> Vid {
        let sub = self.sub_at(vid);
        sub.next(vid, dir)
    }

    /// Get a point.
    ///
    /// * `vid` Vertex ID.
    fn point(&self, vid: Vid) -> WidePt {
        self.points[usize::from(vid)]
    }

    /// Add a point.
    ///
    /// * `pt` Point to add (w indicates stroke width).
    pub fn add_point(&mut self, pt: WidePt) {
        let n_pts = self.points.len();
        if n_pts < usize::from(Vid::MAX) {
            let done = self.sub_current().done;
            if done {
                self.sub_add();
            }
            if done || !self.coincident(pt) {
                self.points.push(pt);
                self.sub_add_point();
            }
        }
    }

    /// Check if a point is coincident with previous point.
    fn coincident(&self, pt: WidePt) -> bool {
        if let Some(p) = self.points.last() {
            pt.0 == p.0
        } else {
            false
        }
    }

    /// Close the current sub-stroke.
    ///
    /// * `joined` If true, join ends of sub-stroke.
    pub fn close(&mut self, joined: bool) {
        if !self.points.is_empty() {
            let sub = self.sub_current();
            sub.joined = joined;
            sub.done = true;
        }
    }

    /// Create path ops of the stroke
    pub fn path_ops(&self) -> Vec<PathOp> {
        // FIXME: this should make a lazy iterator
        let mut ops = vec![];
        let n_subs = self.len();
        for i in 0..n_subs {
            self.stroke_sub(&mut ops, i);
        }
        ops
    }

    /// Stroke one sub-figure.
    fn stroke_sub(&self, ops: &mut Vec<PathOp>, i: usize) {
        if self.sub_points(i) > Vid(0) {
            let start = self.sub_start(i);
            let end = self.sub_end(i);
            let joined = self.sub_joined(i);
            self.stroke_side(ops, i, start, Dir::Forward);
            if joined {
                ops.push(PathOp::Close());
            }
            self.stroke_side(ops, i, end, Dir::Reverse);
            ops.push(PathOp::Close());
        }
    }

    /// Stroke one side of a sub-figure to another figure.
    fn stroke_side(
        &self,
        ops: &mut Vec<PathOp>,
        i: usize,
        start: Vid,
        dir: Dir,
    ) {
        let mut xr: Option<(Pt<f32>, Pt<f32>)> = None;
        let mut v0 = start;
        let mut v1 = self.next(v0, dir);
        let joined = self.sub_joined(i);
        for _ in 0..usize::from(self.sub_points(i)) {
            let p0 = self.point(v0);
            let p1 = self.point(v1);
            let bounds = self.stroke_offset(p0, p1);
            let (pr0, pr1) = bounds;
            if let Some((xr0, xr1)) = xr {
                self.stroke_join(ops, p0, xr0, xr1, pr0, pr1);
            } else if !joined {
                self.stroke_point(ops, pr0);
            }
            xr = Some(bounds);
            v0 = v1;
            v1 = self.next(v1, dir);
        }
        if !joined {
            if let Some((_, xr1)) = xr {
                self.stroke_point(ops, xr1);
            }
        }
    }

    /// Offset segment by half stroke width.
    ///
    /// * `p0` First point.
    /// * `p1` Second point.
    fn stroke_offset(&self, p0: WidePt, p1: WidePt) -> (Pt<f32>, Pt<f32>) {
        // FIXME: scale offset to allow user units as well as pixel units
        let pp0 = p0.0;
        let pp1 = p1.0;
        let vr = (pp1 - pp0).right().normalize();
        let pr0 = pp0 + vr * (p0.w() / 2.0);
        let pr1 = pp1 + vr * (p1.w() / 2.0);
        (pr0, pr1)
    }

    /// Add a point to stroke figure.
    fn stroke_point(&self, ops: &mut Vec<PathOp>, pt: Pt<f32>) {
        ops.push(PathOp::Line(pt));
    }

    /// Add a stroke join.
    ///
    /// * `p` Join point (with stroke width).
    /// * `a0` First point of A segment.
    /// * `a1` Second point of A segment.
    /// * `b0` First point of B segment.
    /// * `b1` Second point of B segment.
    fn stroke_join(
        &self,
        ops: &mut Vec<PathOp>,
        p: WidePt,
        a0: Pt<f32>,
        a1: Pt<f32>,
        b0: Pt<f32>,
        b1: Pt<f32>,
    ) {
        match self.join_style {
            JoinStyle::Miter(ml) => self.stroke_miter(ops, a0, a1, b0, b1, ml),
            JoinStyle::Bevel => self.stroke_bevel(ops, a1, b0),
            JoinStyle::Round => self.stroke_round(ops, p, a0, a1, b0, b1),
        }
    }

    /// Add a miter join.
    fn stroke_miter(
        &self,
        ops: &mut Vec<PathOp>,
        a0: Pt<f32>,
        a1: Pt<f32>,
        b0: Pt<f32>,
        b1: Pt<f32>,
        ml: f32,
    ) {
        // formula: miter_length / stroke_width = 1 / sin ( theta / 2 )
        //      so: stroke_width / miter_length = sin ( theta / 2 )
        if ml > 0.0 {
            // Minimum stroke:miter ratio
            let sm_min = 1.0 / ml;
            let th = (a1 - a0).angle_rel(b0 - b1);
            let sm = (th / 2.0).sin().abs();
            if sm >= sm_min && sm < 1.0 {
                let lna = Line::new(a0, a1);
                let lnb = Line::new(b0, b1);
                // Calculate miter point
                if let Some(xp) = lna.intersection(lnb) {
                    self.stroke_point(ops, xp);
                    return;
                }
            }
        }
        self.stroke_bevel(ops, a1, b0);
    }

    /// Add a bevel join.
    fn stroke_bevel(&self, ops: &mut Vec<PathOp>, a1: Pt<f32>, b0: Pt<f32>) {
        self.stroke_point(ops, a1);
        self.stroke_point(ops, b0);
    }

    /// Add a round join.
    ///
    /// * `p` Join point (with stroke width).
    /// * `a1` Second point of A segment.
    /// * `b0` First point of B segment.
    fn stroke_round(
        &self,
        ops: &mut Vec<PathOp>,
        p: WidePt,
        a0: Pt<f32>,
        a1: Pt<f32>,
        b0: Pt<f32>,
        b1: Pt<f32>,
    ) {
        let th = (a1 - a0).angle_rel(b0 - b1);
        if th <= 0.0 {
            self.stroke_bevel(ops, a1, b0);
        } else {
            self.stroke_point(ops, a1);
            self.stroke_arc(ops, p, a1, b0);
        }
    }

    /// Add a stroke arc.
    fn stroke_arc(
        &self,
        ops: &mut Vec<PathOp>,
        p: WidePt,
        a: Pt<f32>,
        b: Pt<f32>,
    ) {
        let p2 = p.0;
        let vr = (b - a).right().normalize();
        let c = p2 + vr * (p.w() / 2.0);
        let ab = a.midpoint(b);
        if self.is_within_tolerance2(c, ab) {
            self.stroke_point(ops, b);
        } else {
            self.stroke_arc(ops, p, a, c);
            self.stroke_arc(ops, p, c, b);
        }
    }
}
