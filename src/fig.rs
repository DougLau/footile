// fig.rs    A 2D rasterizer.
//
// Copyright (c) 2017-2018  Douglas P Lau
//
use std::cmp;
use std::cmp::Ordering;
use std::cmp::Ordering::*;
use std::fmt;
use std::ops;
use geom::Vec2;
use mask::Mask;
use path::FillRule;

/// Vertex ID
type Vid = u16;

/// Fixed-point type for fast calculations
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Fixed {
    v: i32,
}

/// Figure direction enum
#[derive(Clone, Copy, PartialEq, Debug)]
enum FigDir {
    Forward,
    Reverse,
}

/// Get the opposite direction
fn opposite(dir: FigDir) -> FigDir {
    match dir {
        FigDir::Forward => FigDir::Reverse,
        FigDir::Reverse => FigDir::Forward,
    }
}

/// Sub-figure structure
struct SubFig {
    start    : Vid,     // starting point
    n_points : Vid,     // number of points
    done     : bool,    // done flag
}

/// Edge structure
#[derive(Debug)]
struct Edge {
    v1       : Vid,         // lower vertex ID
    y0f      : Option<i32>, // fractional Y at upper vertex
    y1f      : Option<i32>, // fractional Y at lower vertex
    dir      : FigDir,      // figure direction from upper to lower
    step_pix : Fixed,       // change in cov per pix on scan line
    islope   : Fixed,       // inverse slope (dx / dy)
    x_bot    : Fixed,       // X at bottom of scan line
    min_x    : Fixed,       // minimum X on scan line
    max_x    : Fixed,       // maximum X on scan line
}

/// A Fig is a series of 2D points which can be rendered to
/// an image [Mask](struct.Mask.html).
///
pub struct Fig {
    points : Vec<Vec2>,         // all points
    subs   : Vec<SubFig>,       // all sub-figures
}

/// Figure scanner structure
struct Scanner<'a> {
    fig      : &'a Fig,         // the figure
    mask     : &'a mut Mask,    // alpha mask
    sgn_area : &'a mut [i16],   // signed area buffer
    edges    : Vec<Edge>,       // active edges
    dir      : FigDir,          // figure direction
    rule     : FillRule,        // fill rule
    y_now    : Fixed,           // current scan Y
    y_prev   : Fixed,           // previous scan Y
    y_bot    : Fixed,           // Y at bottom of mask
}

/// Number of bits at fixed point
const FRAC_BITS: i32 = 16;

/// Mask of fixed fractional bits
const FRAC_MASK: i32 = ((1 << FRAC_BITS) - 1);

/// Fixed-point constants
const FX_ZERO: Fixed = Fixed { v: 0 };
const FX_ONE: Fixed = Fixed { v: 1 << FRAC_BITS };
const FX_HALF: Fixed = Fixed { v: 1 << (FRAC_BITS - 1) };
const FX_EPSILON: Fixed = Fixed { v: 1 };

impl fmt::Debug for Fixed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_f32())
    }
}

impl ops::Add for Fixed {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        if cfg!(saturating_fixed) {
            return Fixed { v: self.v.saturating_add(other.v) };
        }
        Fixed { v: self.v + other.v }
    }
}

impl ops::Sub for Fixed {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        if cfg!(saturating_fixed) {
            return Fixed { v: self.v.saturating_sub(other.v) };
        }
        Fixed { v: self.v - other.v }
    }
}

impl ops::Mul for Fixed {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        let v: i64 = (self.v as i64 * other.v as i64) >> FRAC_BITS;
        if cfg!(saturating_fixed) {
            if v > i32::max_value() as i64 {
                return Fixed { v: i32::max_value() };
            } else if v < i32::min_value() as i64 {
                return Fixed { v: i32::min_value() };
            }
        }
        Fixed { v: v as i32 }
    }
}

impl ops::Div for Fixed {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        let v = ((self.v as i64) << (FRAC_BITS as i64)) / other.v as i64;
        if cfg!(saturating_fixed) {
            if v > i32::max_value() as i64 {
                return Fixed { v: i32::max_value() };
            } else if v < i32::min_value() as i64 {
                return Fixed { v: i32::min_value() };
            }
        }
        Fixed { v: v as i32 }
    }
}

impl Fixed {
    fn from_i32(v: i32) -> Fixed {
        Fixed { v: v << FRAC_BITS }
    }
    fn from_f32(v: f32) -> Fixed {
        Fixed { v: (v * (FX_ONE.v as f32)) as i32 }
    }
    fn to_i32(self) -> i32 {
        self.v >> FRAC_BITS
    }
    fn to_f32(self) -> f32 {
        self.v as f32 / (FX_ONE.v as f32)
    }
    fn abs(self) -> Fixed {
        Fixed { v: self.v.abs() }
    }
    fn floor(self) -> Fixed {
        Fixed { v: self.v & !FRAC_MASK }
    }
    fn ceil(self) -> Fixed {
        (self - FX_EPSILON).floor() + FX_ONE
    }
    fn frac(self) -> Fixed {
        Fixed { v: self.v & FRAC_MASK }
    }
    fn avg(self, other: Fixed) -> Fixed {
        let v = self.v + other.v >> 1;
        Fixed { v: v }
    }
    /// Compare two f32 for fixed-point equality
    fn cmp_f32(a: f32, b: f32) -> Ordering {
        Fixed::from_f32(a).v.cmp(&Fixed::from_f32(b).v)
    }
    /// Get the line of a value
    fn line_of(self) -> i32 {
        (self - FX_EPSILON).to_i32()
    }
}

impl SubFig {
    /// Create a new sub-figure
    fn new(start: Vid) -> SubFig {
        SubFig {
            start    : start,
            n_points : 0 as Vid,
            done     : false,
        }
    }
    /// Get next vertex within a sub-figure
    fn next(&self, vid: Vid, dir: FigDir) -> Vid {
        match dir {
            FigDir::Forward => {
                let v = vid + 1 as Vid;
                if v < self.start + self.n_points {
                    v
                } else {
                    self.start
                }
            },
            FigDir::Reverse => {
                if vid > self.start {
                    vid - 1 as Vid
                } else {
                    self.start + self.n_points - 1 as Vid
                }
            },
        }
    }
}

impl Edge {
    /// Create a new edge
    fn new(v0: Vid, v1: Vid, p0: Vec2, p1: Vec2, dir: FigDir) -> Edge {
        assert!(v0 != v1);
        let dx = Fixed::from_f32(p1.x - p0.x);  // delta X
        let dy = Fixed::from_f32(p1.y - p0.y);  // delta Y
        assert!(dy > FX_ZERO);
        let step_pix = Edge::calculate_step(dx, dy);
        let islope = dx / dy;
        let y0 = Fixed::from_f32(p0.y);
        let y1 = Fixed::from_f32(p1.y);
        let y0f = if y0.frac() > FX_ZERO { Some(y0.to_i32()) } else { None };
        let y1f = if y1.frac() > FX_ZERO { Some(y1.to_i32()) } else { None };
        let fm = (y0.ceil() - y0) * islope;
        let x_bot = fm + Fixed::from_f32(p0.x);
        Edge {
            v1       : v1,
            y0f      : y0f,
            y1f      : y1f,
            dir      : dir,
            step_pix : step_pix,
            islope   : islope,
            x_bot    : x_bot,
            min_x    : FX_ZERO,
            max_x    : FX_ZERO,
        }
    }
    /// Calculate the step for each pixel on an edge
    fn calculate_step(dx: Fixed, dy: Fixed) -> Fixed {
        if dx != FX_ZERO {
            cmp::min(FX_ONE, (dy / dx).abs())
        } else {
            FX_ZERO
        }
    }
    /// Check if edge is partial at a given row.
    fn is_partial(&self, y: i32) -> bool {
        (if let Some(y0) = self.y0f { y == y0 } else { false }) ||
        (if let Some(y1) = self.y1f { y == y1 } else { false })
    }
    /// Calculate X limits for a partial scan line
    fn calculate_x_limits_partial(&mut self, ypb: Fixed, ynb: Fixed) {
        let xt = self.x_bot - self.islope * ypb;
        let xb = self.x_bot - self.islope * ynb;
        self.set_x_limits(xt, xb);
    }
    /// Calculate X limits for a full scan line
    fn calculate_x_limits_full(&mut self) {
        let xt = self.x_bot - self.islope;
        let xb = self.x_bot;
        self.set_x_limits(xt, xb);
    }
    /// Set X limits
    fn set_x_limits(&mut self, xt: Fixed, xb: Fixed) {
        self.min_x = cmp::min(xt, xb);
        self.max_x = cmp::max(xt, xb);
    }
    /// Get the minimum X pixel
    fn min_pix(&self) -> i32 {
        self.min_x.to_i32()
    }
    /// Get the maximum X pixel
    fn max_pix(&self) -> i32 {
        self.max_x.to_i32()
    }
    /// Get coverage of first pixel on edge.
    fn first_cov(&self) -> Fixed {
        let r = if self.min_pix() == self.max_pix() {
            (FX_ONE - self.x_mid().frac())
        } else {
            (FX_ONE - self.min_x.frac()) * FX_HALF
        };
        self.step_cov(r)
    }
    /// Get pixel coverage.
    fn step_cov(&self, r: Fixed) -> Fixed {
        if self.step_pix > FX_ZERO {
            r * self.step_pix
        } else {
            r
        }
    }
    /// Get the X midpoint for the current scan line
    fn x_mid(&self) -> Fixed {
        self.max_x.avg(self.min_x)
    }
    /// Scan signed area of edge
    fn scan_area(&self, dir: FigDir, cov_full: i16, area: &mut [i16]) {
        let w = area.len() as i32;
        let ed = if self.dir == dir { 1i16 } else { -1i16 };
        let s_0 = pixel_cov(self.first_cov());
        let s_n = pixel_cov(self.step_cov(FX_ONE));
        assert!(s_n > 0);
        let mut cc = s_0;
        let mut cov = 0i16;
        let mut x = self.min_pix();
        while x < w && cov < cov_full {
            let c = cmp::min(cc, cov_full - cov);
            cov += c;
            area[cmp::max(0, x) as usize] += c * ed;
            cc = s_n;
            x += 1;
        }
    }
}

impl fmt::Debug for Fig {
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

impl Fig {
    /// Create a figure rasterizer
    pub fn new() -> Fig {
        let points = Vec::with_capacity(1024);
        let mut subs = Vec::with_capacity(16);
        subs.push(SubFig::new(0 as Vid));
        Fig { points, subs }
    }
    /// Get the current sub-figure
    fn sub_current(&mut self) -> &mut SubFig {
        self.subs.last_mut().unwrap()
    }
    /// Add a new sub-figure
    fn sub_add(&mut self) {
        let vid = self.points.len() as Vid;
        self.subs.push(SubFig::new(vid));
    }
    /// Add a point to the current sub-figure
    fn sub_add_point(&mut self) {
        let sub = self.sub_current();
        sub.n_points += 1;
    }
    /// Get the sub-figure at a specified vertex ID
    fn sub_at(&self, vid: Vid) -> &SubFig {
        for sub in self.subs.iter() {
            if vid < sub.start + sub.n_points {
                return sub;
            }
        }
        // Invalid vid indicates bug
        unreachable!();
    }
    /// Get next vertex
    fn next(&self, vid: Vid, dir: FigDir) -> Vid {
        let sub = self.sub_at(vid);
        sub.next(vid, dir)
    }
    /// Get the next vertex with a different Y
    fn next_y(&self, vid: Vid, dir: FigDir) -> Vid {
        let py = self.get_y(vid);
        let sub = self.sub_at(vid);
        let mut v = sub.next(vid, dir);
        while v != vid {
            let y = self.get_y(v);
            if Fixed::cmp_f32(py, y) != Equal {
                return v;
            }
            v = sub.next(v, dir);
        }
        vid
    }
    /// Get the next vertex for an edge change
    fn next_edge(&self, vid: Vid, dir: FigDir) -> Vid {
        let pp = self.get_point(vid);
        let sub = self.sub_at(vid);
        let mut v = sub.next(vid, dir);
        while v != vid {
            let p = self.get_point(v);
            if p.x < pp.x || Fixed::cmp_f32(pp.y, p.y) != Equal {
                return v;
            }
            v = sub.next(v, dir);
        }
        vid
    }
    /// Get the last vertex with the same Y
    fn same_y(&self, vid: Vid, dir: FigDir) -> Vid {
        let py = self.get_y(vid);
        let sub = self.sub_at(vid);
        let mut vp = vid;
        let mut v = sub.next(vid, dir);
        while v != vid {
            let y = self.get_y(v);
            if Fixed::cmp_f32(py, y) != Equal {
                return vp;
            }
            vp = v;
            v = sub.next(v, dir);
        }
        vid
    }
    /// Get direction from top vertex.
    fn get_dir(&self, vid: Vid) -> FigDir {
        let p0 = self.get_point(self.next(vid, FigDir::Forward));
        let p1 = self.get_point(self.next(vid, FigDir::Reverse));
        if p0.x < p1.x {
            FigDir::Forward
        } else {
            FigDir::Reverse
        }
    }
    /// Get a point.
    ///
    /// * `vid` Vertex ID.
    fn get_point(&self, vid: Vid) -> Vec2 {
        self.points[vid as usize]
    }
    /// Get Y value at a vertex.
    fn get_y(&self, vid: Vid) -> f32 {
        self.get_point(vid).y
    }
    /// Add a point.
    ///
    /// * `pt` Point to add.
    pub fn add_point(&mut self, pt: Vec2) {
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
    fn coincident(&self, pt: Vec2) -> bool {
        if let Some(p) = self.points.last() {
            pt == *p
        } else {
            false
        }
    }
    /// Close the current sub-figure.
    pub fn close(&mut self, _joined: bool) {
        if self.points.len() > 0 {
            let sub = self.sub_current();
            sub.done = true;
        }
    }
    /// Compare two figure vertex IDs
    fn compare_vids(&self, v0: Vid, v1: Vid) -> Ordering {
        let p0 = self.get_point(v0);
        let p1 = self.get_point(v1);
        match Fixed::cmp_f32(p0.y, p1.y) {
            Less    => Less,
            Greater => Greater,
            Equal   => {
                p0.x.partial_cmp(&p1.x).unwrap_or(Equal)
            },
        }
    }
    /// Fill the figure to an image mask.
    ///
    /// * `mask` Output mask.
    /// * `sgn_area` Signed area buffer.
    /// * `rule` Fill rule.
    pub fn fill(&self, mask: &mut Mask, sgn_area: &mut [i16], rule: FillRule) {
        let n_points = self.points.len() as Vid;
        let mut vids: Vec<Vid> = (0 as Vid..n_points).collect();
        vids.sort_by(|a,b| self.compare_vids(*a, *b));
        let dir = self.get_dir(vids[0]);
        let mut scan = Scanner::new(self, mask, sgn_area, dir, rule);
        for vid in vids {
            if scan.is_complete() {
                break;
            }
            scan.scan_vertex(vid);
        }
        scan.scan_accumulate();
    }
}

impl<'a> Scanner<'a> {
    /// Create a new figure scanner struct
    fn new(fig: &'a Fig, mask: &'a mut Mask, sgn_area: &'a mut [i16],
           dir: FigDir, rule: FillRule) -> Scanner<'a>
    {
        assert!(mask.width() <= sgn_area.len() as u32);
        let y_bot = Fixed::from_i32(mask.height() as i32);
        let edges = Vec::with_capacity(16);
        Scanner {
            fig      : fig,
            mask     : mask,
            sgn_area : sgn_area,
            edges    : edges,
            dir      : dir,
            rule     : rule,
            y_now    : FX_ZERO,
            y_prev   : FX_ZERO,
            y_bot    : y_bot,
        }
    }
    /// Scan figure to a given vertex
    fn scan_vertex(&mut self, vid: Vid) {
        let y = self.get_y(vid);
        let y_vtx = Fixed::from_f32(y);
        if self.edges.len() > 0 {
            self.scan_to_y(y_vtx);
        } else {
            self.y_now = y_vtx;
            self.y_prev = y_vtx;
        }
        self.update_edges(vid);
    }
    /// Get Y value at a vertex.
    fn get_y(&self, vid: Vid) -> f32 {
        self.fig.get_y(vid)
    }
    /// Scan figure, rasterizing all lines above a vertex
    fn scan_to_y(&mut self, y_vtx: Fixed) {
        while self.y_now < y_vtx && !self.is_complete() {
            if self.is_line_bottom() {
                self.scan_accumulate();
            }
            self.y_prev = self.y_now;
            self.y_now = cmp::min(y_vtx, self.y_now.floor() + FX_ONE);
            if self.is_next_line() {
                self.advance_edges();
            }
            if self.y_now > FX_ZERO {
                if self.is_partial() {
                    self.scan_partial();
                }
                if self.is_line_bottom() {
                    self.scan_full();
                }
            }
        }
    }
    /// Check if scan is complete (reached bottom of mask)
    fn is_complete(&self) -> bool {
        self.y_now.line_of() >= self.y_bot.line_of()
    }
    /// Check if scan is at bottom of line
    fn is_line_bottom(&self) -> bool {
        self.y_now.frac() == FX_ZERO
    }
    /// Check if scan has advanced to the next line
    fn is_next_line(&self) -> bool {
        self.y_now.line_of() > self.y_prev.line_of()
    }
    /// Advance all edges to the next line
    fn advance_edges(&mut self) {
        for e in self.edges.iter_mut() {
            e.x_bot = e.x_bot + e.islope;
        }
    }
    /// Check if current scan line is partial
    fn is_partial(&self) -> bool {
        (self.y_now - self.y_prev) < FX_ONE
    }
    /// Scan partial edges
    fn scan_partial(&mut self) {
        let cov_full = self.scan_coverage();
        assert!(cov_full <= 256i16);
        if cov_full <= 0i16 {
            return;
        }
        let y = self.y_now.line_of();
        let y_bot = self.y_now.ceil();
        let ypb = y_bot - self.y_prev;
        let ynb = y_bot - self.y_now;
        let mut area = &mut self.sgn_area;
        for e in self.edges.iter_mut() {
            if e.is_partial(y) {
                e.calculate_x_limits_partial(ypb, ynb);
                e.scan_area(self.dir, cov_full, &mut area);
            }
        }
    }
    /// Scan full edges.
    fn scan_full(&mut self) {
        let y = self.y_now.line_of();
        let mut area = &mut self.sgn_area;
        for e in self.edges.iter_mut() {
            if !e.is_partial(y) {
                e.calculate_x_limits_full();
                e.scan_area(self.dir, 256i16, &mut area);
            }
        }
    }
    /// Accumulate signed area to mask.
    fn scan_accumulate(&mut self) {
        if self.y_now > FX_ZERO {
            let y = self.y_now.line_of() as u32;
            self.mask.scan_accumulate(self.sgn_area, y, self.rule);
        }
    }
    /// Get full scan coverage
    fn scan_coverage(&self) -> i16 {
        assert!(self.y_now > self.y_prev);
        assert!(self.y_now <= self.y_prev + FX_ONE);
        let scan_now = pixel_cov(self.y_now.frac());
        let scan_prev = pixel_cov(self.y_prev.frac());
        if scan_now == scan_prev && self.y_now.frac() > FX_ZERO {
            0
        } else if scan_now > scan_prev {
            scan_now - scan_prev
        } else {
            256 + scan_now - scan_prev
        }
    }
    /// Update edges at a given vertex
    fn update_edges(&mut self, vid: Vid) {
        let vp = self.fig.next_edge(vid, FigDir::Reverse);
        let vn = self.fig.next_edge(vid, FigDir::Forward);
        if (vp != vid) && (vn != vid) {
            let y = self.get_y(vid);
            let cp = Fixed::cmp_f32(self.get_y(vp), y);
            let cn = Fixed::cmp_f32(self.get_y(vn), y);
            match (cp, cn) {
                (Less,    Less)    => self.edge_merge(vid),
                (Greater, Greater) => self.edge_split(vp, vn),
                _                  => self.edge_regular(vid),
            }
        }
    }
    /// Remove two edges at a merge vertex
    fn edge_merge(&mut self, vid: Vid) {
        let fig = &self.fig;
        let mut i = self.edges.len();
        while i > 0 {
            i -= 1;
            let v1 = self.edges[i].v1;
            if (fig.same_y(v1, FigDir::Forward) == vid) ||
               (fig.same_y(v1, FigDir::Reverse) == vid)
            {
                self.edges.remove(i);
            }
        }
    }
    /// Add two edges at a split vertex
    fn edge_split(&mut self, v0: Vid, v1: Vid) {
        let fig = &self.fig;
        let v0u = fig.next(v0, FigDir::Forward);    // Find upper vtx of edge 0
        let p0u = fig.get_point(v0u);               // Upper point of edge 0
        let p0 = fig.get_point(v0);                 // Lower point of edge 0
        self.edges.push(Edge::new(v0u, v0, p0u, p0, FigDir::Reverse));
        let v1u = fig.next(v1, FigDir::Reverse);    // Find upper vtx of edge 1
        let p1u = fig.get_point(v1u);               // Upper point of edge 1
        let p1 = fig.get_point(v1);                 // Lower point of edge 1
        self.edges.push(Edge::new(v1u, v1, p1u, p1, FigDir::Forward));
    }
    /// Update one edge at a regular vertex
    fn edge_regular(&mut self, vid: Vid) {
        let fig = &self.fig;
        for e in self.edges.iter_mut() {
            if vid == e.v1 {
                let dir = e.dir;
                let vn = fig.next_y(vid, dir);          // Find lower vertex
                let v = fig.next_y(vn, opposite(dir));  // Find upper vertex
                let p = fig.get_point(v);
                let pn = fig.get_point(vn);
                *e = Edge::new(v, vn, p, pn, dir);
                break;
            }
        }
    }
}

/// Calculate pixel coverage
///
/// fcov Total coverage (0 to 1 fixed-point).
/// return Total pixel coverage (0 to 256).
fn pixel_cov(fcov: Fixed) -> i16 {
    assert!(fcov >= FX_ZERO && fcov <= FX_ONE);
    // Round to nearest cov value
    let n = (fcov.v + (1 << 7)) >> 8;
    n as i16
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_fixed() {
        let a = Fixed::from_i32(37);
        let b = Fixed::from_i32(3);
        let c = Fixed::from_f32(1.5f32);
        let d = Fixed::from_f32(-2.5f32);
        let e = Fixed::from_i32(128);
        assert!(a.to_i32() == 37);
        assert!(a.floor() == a);
        assert!(a.ceil() == a);
        assert!(a * b == Fixed::from_i32(111));
        assert!(a / b == Fixed::from_f32(12.33333f32));
        assert!(b.floor() == b);
        assert!(b.ceil() == b);
        assert!(c.floor() == Fixed::from_i32(1));
        assert!(c.ceil() == Fixed::from_i32(2));
        assert!(c.frac() == Fixed::from_f32(0.5f32));
        assert!(d.abs() == Fixed::from_f32(2.5f32));
        assert!(d.frac() == Fixed::from_f32(0.5f32));
        assert!(d.floor() == Fixed::from_i32(-3));
        assert!(d.ceil() == Fixed::from_i32(-2));
        assert!(cmp::min(a, b) == b);
        assert!(cmp::max(a, b) == a);
        assert!(a.avg(b) == Fixed::from_i32(20));
        assert!(b.avg(c) == Fixed::from_f32(2.25f32));
        assert!((e * e).to_i32() == 16384);
        assert!(Fixed::cmp_f32(0f32, 0f32) == Ordering::Equal);
        assert!(Fixed::cmp_f32(0f32, 0.00001f32) == Ordering::Equal);
        assert!(Fixed::cmp_f32(0f32, 0.0001f32) == Ordering::Less);
        assert!(Fixed::cmp_f32(0f32, -0.0001f32) == Ordering::Greater);
    }
    #[test]
    fn fig_3x3() {
        let mut m = Mask::new(3, 3);
        let mut s = vec!(0; 3);
        let mut f = Fig::new();
        f.add_point(Vec2::new(0f32, 0f32));
        f.add_point(Vec2::new(3f32, 3f32));
        f.add_point(Vec2::new(0f32, 3f32));
        f.fill(&mut m, &mut s, FillRule::NonZero);
        let p: Vec<_> = m.iter().cloned().collect();
        assert!(p == [128, 0, 0, 255, 128, 0, 255, 255, 128]);
    }
    #[test]
    fn fig_9x1() {
        let mut m = Mask::new(9, 1);
        let mut s = vec!(0; 16);
        let mut f = Fig::new();
        f.add_point(Vec2::new(0f32, 0f32));
        f.add_point(Vec2::new(9f32, 1f32));
        f.add_point(Vec2::new(0f32, 1f32));
        f.fill(&mut m, &mut s, FillRule::NonZero);
        let p: Vec<_> = m.iter().cloned().collect();
        assert!(p == [242, 214, 186, 158, 130, 102, 74, 46, 18]);
    }
}
