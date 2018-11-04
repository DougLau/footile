// stroker.rs   A path stroker.
//
// Copyright (c) 2017-2018  Douglas P Lau
//
use std::fmt;
use fig::Fig;
use geom::{Vec2, Vec2w, intersection};
use path::JoinStyle;

/// Vertex ID
type Vid = u16;

/// Stroke direction enum
#[derive(Clone, Copy, PartialEq, Debug)]
enum Dir {
    Forward,
    Reverse,
}

/// Sub-stroke struct
struct SubStroke {
    start    : Vid,     // starting point
    n_points : Vid,     // number of points
    joined   : bool,    // joined ends flag
    done     : bool,    // done flag
}

/// Stroke struct
pub struct Stroke {
    join_style : JoinStyle,      // join style
    tol_sq     : f32,            // tolerance squared
    points     : Vec<Vec2w>,     // all points
    subs       : Vec<SubStroke>, // all sub-strokes
}

impl SubStroke {
    /// Create a new sub-stroke
    fn new(start: Vid) -> SubStroke {
        SubStroke {
            start    : start,
            n_points : 0 as Vid,
            joined   : false,
            done     : false,
        }
    }
    /// Get next vertex within a sub-stroke
    fn next(&self, vid: Vid, dir: Dir) -> Vid {
        match dir {
            Dir::Forward => {
                let v = vid + 1 as Vid;
                if v < self.start + self.n_points {
                    v
                } else {
                    self.start
                }
            },
            Dir::Reverse => {
                if vid > self.start {
                    vid - 1 as Vid
                } else {
                    self.start + self.n_points - 1 as Vid
                }
            },
        }
    }
    /// Get count of points
    fn count(&self) -> Vid {
        if self.joined {
            self.n_points + 1 as Vid
        } else if self.n_points > 0 {
            self.n_points - 1 as Vid
        } else {
            0 as Vid
        }
    }
}

impl fmt::Debug for Stroke {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for sub in &self.subs {
            write!(f, "sub {}+{} ", sub.start, sub.n_points)?;
            for v in sub.start..(sub.start + sub.n_points) {
                write!(f, "{:?} ", self.get_point(v))?;
            }
        }
        Ok(())
    }
}

impl Stroke {
    pub fn new(join_style: JoinStyle, tol_sq: f32) -> Stroke {
        let points = Vec::with_capacity(1024);
        let mut subs = Vec::with_capacity(16);
        subs.push(SubStroke::new(0 as Vid));
        Stroke { join_style, tol_sq, points, subs }
    }
    /// Check if two points are within tolerance threshold.
    fn is_within_tolerance2(&self, a: Vec2, b: Vec2) -> bool {
        assert!(self.tol_sq > 0f32);
        a.dist_sq(b) <= self.tol_sq
    }
    /// Get the count of sub-strokes
    fn sub_count(&self) -> usize {
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
        self.subs[i].count()
    }
    /// Get the current sub-stroke
    fn sub_current(&mut self) -> &mut SubStroke {
        let len = self.subs.len();
        &mut self.subs[len - 1]
    }
    /// Add a new sub-stroke
    fn sub_add(&mut self) {
        let vid = self.points.len() as Vid;
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
    fn get_point(&self, vid: Vid) -> Vec2w {
        self.points[vid as usize]
    }
    /// Add a point.
    ///
    /// * `pt` Point to add (z indicates stroke width).
    pub fn add_point(&mut self, pt: Vec2w) {
        let n_pts = self.points.len();
        if n_pts < Vid::max_value() as usize {
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
    fn coincident(&self, pt: Vec2w) -> bool {
        let n = self.points.len();
        if n > 0 {
            let p = self.points[n - 1];
            p.v == pt.v
        } else {
            false
        }
    }
    /// Close the current sub-stroke.
    ///
    /// * `joined` If true, join ends of sub-stroke.
    pub fn close(&mut self, joined: bool) {
        if self.points.len() > 0 {
            let sub = self.sub_current();
            sub.joined = joined;
            sub.done = true;
        }
    }
    /// Create a fig of the stroke
    pub fn to_fig(&self) -> Fig {
        let mut fig = Fig::new();
        let n_subs = self.sub_count();
        for i in 0..n_subs {
            self.stroke_sub(&mut fig, i);
        }
        fig
    }
    /// Stroke one sub-figure.
    fn stroke_sub(&self, fig: &mut Fig, i: usize) {
        if self.sub_points(i) > 0 {
            let start = self.sub_start(i);
            let end = self.sub_end(i);
            let joined = self.sub_joined(i);
            self.stroke_side(fig, i, start, Dir::Forward);
            if joined {
                fig.close(true);
            }
            self.stroke_side(fig, i, end, Dir::Reverse);
            fig.close(joined);
        }
    }
    /// Stroke one side of a sub-figure to another figure.
    fn stroke_side(&self, fig: &mut Fig, i: usize, start: Vid, dir: Dir) {
        let mut xr: Option<(Vec2, Vec2)> = None;
        let mut v0 = start;
        let mut v1 = self.next(v0, dir);
        let joined = self.sub_joined(i);
        for _ in 0..self.sub_points(i) {
            let p0 = self.get_point(v0);
            let p1 = self.get_point(v1);
            let bounds = self.stroke_offset(p0, p1);
            let (pr0, pr1) = bounds;
            if let Some((xr0, xr1)) = xr {
                self.stroke_join(fig, p0, xr0, xr1, pr0, pr1);
            } else if !joined {
                self.stroke_point(fig, pr0);
            }
            xr = Some(bounds);
            v0 = v1;
            v1 = self.next(v1, dir);
        }
        if !joined {
            if let Some((_, xr1)) = xr {
                self.stroke_point(fig, xr1);
            }
        }
    }
    /// Offset segment by half stroke width.
    ///
    /// * `p0` First point.
    /// * `p1` Second point.
    fn stroke_offset(&self, p0: Vec2w, p1: Vec2w) -> (Vec2, Vec2) {
        // FIXME: scale offset to allow user units as well as pixel units
        let pp0 = p0.v;
        let pp1 = p1.v;
        let vr = (pp1 - pp0).right().normalize();
        let pr0 = pp0 + vr * (p0.w / 2f32);
        let pr1 = pp1 + vr * (p1.w / 2f32);
        (pr0, pr1)
    }
    /// Add a point to stroke figure.
    fn stroke_point(&self, fig: &mut Fig, pt: Vec2) {
        fig.add_point(Vec2w::new(pt.x, pt.y, 1f32));
    }
    /// Add a stroke join.
    ///
    /// * `p` Join point (with stroke width).
    /// * `a0` First point of A segment.
    /// * `a1` Second point of A segment.
    /// * `b0` First point of B segment.
    /// * `b1` Second point of B segment.
    fn stroke_join(&self, fig: &mut Fig, p: Vec2w, a0: Vec2, a1: Vec2,
        b0: Vec2, b1: Vec2)
    {
        match self.join_style {
            JoinStyle::Miter(ml) => self.stroke_miter(fig, a0, a1, b0, b1, ml),
            JoinStyle::Bevel     => self.stroke_bevel(fig, a1, b0),
            JoinStyle::Round     => self.stroke_round(fig, p, a0, a1, b0, b1),
        }
    }
    /// Add a miter join.
    fn stroke_miter(&self, fig: &mut Fig, a0: Vec2, a1: Vec2, b0: Vec2,
        b1: Vec2, ml: f32)
    {
        // formula: miter_length / stroke_width = 1 / sin ( theta / 2 )
        //      so: stroke_width / miter_length = sin ( theta / 2 )
        if ml > 0f32 {
            // Minimum stroke:miter ratio
            let sm_min = 1f32 / ml;
            let th = (a1 - a0).angle_rel(b0 - b1);
            let sm = (th / 2f32).sin().abs();
            if sm >= sm_min && sm < 1f32 {
                // Calculate miter point
                if let Some(xp) = intersection(a0, a1, b0, b1) {
                    self.stroke_point(fig, xp);
                    return;
                }
            }
        }
        self.stroke_bevel(fig, a1, b0);
    }
    /// Add a bevel join.
    fn stroke_bevel(&self, fig: &mut Fig, a1: Vec2, b0: Vec2) {
        self.stroke_point(fig, a1);
        self.stroke_point(fig, b0);
    }
    /// Add a round join.
    ///
    /// * `p` Join point (with stroke width).
    /// * `a1` Second point of A segment.
    /// * `b0` First point of B segment.
    fn stroke_round(&self, fig: &mut Fig, p: Vec2w, a0: Vec2, a1: Vec2,
        b0: Vec2, b1: Vec2)
    {
        let th = (a1 - a0).angle_rel(b0 - b1);
        if th <= 0f32 {
            self.stroke_bevel(fig, a1, b0);
        } else {
            self.stroke_point(fig, a1);
            self.stroke_arc(fig, p, a1, b0);
        }
    }
    /// Add a stroke arc.
    fn stroke_arc(&self, fig: &mut Fig, p: Vec2w, a: Vec2, b: Vec2) {
        let p2 = p.v;
        let vr = (b - a).right().normalize();
        let c = p2 + vr * (p.w / 2f32);
        let ab = a.midpoint(b);
        if self.is_within_tolerance2(c, ab) {
            self.stroke_point(fig, b);
        } else {
            self.stroke_arc(fig, p, a, c);
            self.stroke_arc(fig, p, c, b);
        }
    }
}
