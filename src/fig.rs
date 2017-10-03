// fig.rs    A 2D rasterizer.
//
// Copyright (c) 2017  Douglas P Lau
//
use std::cmp;
use std::cmp::Ordering;
use std::cmp::Ordering::*;
use std::fmt;
use std::ops;
use super::geom::Vec2;
use super::geom::Vec3;
use super::mask::Mask;

/// Fixed-point type for fast calculations
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Fixed {
	v: i32,
}

/// Figure direction enum
#[derive(Clone, Copy)]
pub enum FigDir {
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

/// Fill-rule for filling figures
#[derive(Debug)]
pub enum FillRule {
    /// All points within bounds are filled
	NonZero,
    /// Alternate filling with figure outline
	EvenOdd,
}

/// Sub-figure structure
struct SubFig {
    start    : u16,     // starting point
    n_points : u16,     // number of points
    joined   : bool,    // joined ends flag
    done     : bool,    // done flag
}

/// Edge structure
struct Edge {
	vtx      : u16,             // lower vertex ID
	dir      : FigDir,          // figure direction from upper to lower
	step_pix : Fixed,           // change in cov per pix on scan line
	islope   : Fixed,           // inverse slope (dx / dy)
	x_bot    : Fixed,           // X at bottom of scan line
	min_x    : Fixed,           // minimum X on scan line
	max_x    : Fixed,           // maximum X on scan line
	min_pix  : i32,             // minimum pixel on scan line
	max_pix  : i32,             // maximum pixel on scan line
}

/// A Fig is a series of 2D points which can be rendered to
/// an image [Mask](struct.Mask.html).
/// It can also be stroked to another figure, which can then be filled.
///
pub struct Fig {
	points : Vec<Vec3>,         // all points
	subs   : Vec<SubFig>,       // all sub-figures
}

/// Figure scanner structure
struct Scanner<'a> {
	fig      : &'a Fig,         // the figure
	mask     : &'a mut Mask,    // alpha mask
	scan_buf : &'a mut Mask,    // scan line buffer
	edges    : Vec<Edge>,       // all edges
	rule     : FillRule,        // fill rule
	n_fill   : i32,             // edge count for fill rule
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

/// Fixed constant one
const FX_ONE: Fixed = Fixed { v: 1 << FRAC_BITS };

/// Fixed epsilon
const FX_EPSILON: Fixed = Fixed { v: 1 };

impl fmt::Debug for Fixed {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{} (0x{:x})", self.to_f32(), self.v)
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
        Fixed::from_f32(a - b).v.cmp(&0i32)
    }
}

impl SubFig {
    /// Create a new sub-figure
    fn new(start: u16) -> SubFig {
        SubFig { start: start, n_points: 0u16, joined: false, done: false }
    }
    /// Get next vertex within a sub-figure
    fn next(&self, vid: u16, dir: FigDir) -> u16 {
        match dir {
            FigDir::Forward => {
                let v = vid + 1u16;
                if v < self.start + self.n_points {
                    v
                } else {
                    self.start
                }
            },
            FigDir::Reverse => {
                if vid > self.start {
                    vid - 1u16
                } else {
                    self.start + self.n_points - 1u16
                }
            },
        }
    }
    /// Get count of points
    fn count(&self) -> u16 {
        if self.joined {
            self.n_points + 1u16
        } else if self.n_points > 0 {
            self.n_points - 1u16
        } else {
            0
        }
    }
}

impl Edge {
    /// Create a new edge
    fn new(v0: u16, v1: u16, p0: &Vec3, p1: &Vec3, dir: FigDir) -> Edge {
        assert!(v0 != v1);
        let dx = Fixed::from_f32(p1.x - p0.x);  // delta X
        let dy = Fixed::from_f32(p1.y - p0.y);  // delta Y
        assert!(dy > FX_ZERO);
        let step_pix = Edge::calculate_step(dx, dy);
        let islope = dx / dy;
        let y = Fixed::from_f32(p0.y);
        let fm = (y.ceil() - y) * islope;
        let x_bot = fm + Fixed::from_f32(p0.x);
        Edge {
            vtx: v1,
            dir: dir,
            step_pix: step_pix,
            islope: islope,
            x_bot: x_bot,
            min_x: FX_ZERO,
            max_x: FX_ZERO,
            min_pix: 0,
            max_pix: 0,
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
    /// Scan one pixel on the edge
    fn scan_pix(&self, x: i32, cov_pix: Fixed) -> Fixed {
        let x_mn = x.cmp(&self.min_pix);
        let x_mx = x.cmp(&self.max_pix);
        match (x_mn, x_mx) {
            (Less, _)          => FX_ZERO,
            (Equal, Equal)     => FX_ONE - self.x_mid().frac(),
            (Equal, _)         => (FX_ONE - self.min_x.frac()) * self.step_pix,
            (Greater, Greater) => FX_ONE,
            (Greater, _)       => cmp::min(FX_ONE, cov_pix + self.step_pix),
        }
    }
    /// Get the X midpoint for the current scan line
    fn x_mid(&self) -> Fixed {
        self.max_x.avg(self.min_x)
    }
}

impl fmt::Debug for Fig {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for sub in &self.subs {
            write!(f, "sub {}+{} ", sub.start, sub.n_points)?;
            for v in sub.start..(sub.start + sub.n_points) {
                let p = &self.points[v as usize];
                write!(f, "{:?} ", p)?;
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
        subs.push(SubFig::new(0u16));
        Fig { points: points, subs: subs }
    }
    /// Get the count of sub-figures
    pub fn sub_count(&self) -> usize {
        self.subs.len()
    }
    /// Get start of a sub-figure
    pub fn sub_start(&self, i: usize) -> u16 {
        self.subs[i].start
    }
    /// Get end of a sub-figure
    pub fn sub_end(&self, i: usize) -> u16 {
        let sub = &self.subs[i];
        sub.next(sub.start, FigDir::Reverse)
    }
    /// Check if a sub-figure is joined
    pub fn sub_joined(&self, i: usize) -> bool {
        self.subs[i].joined
    }
    /// Get the number of points in a sub-figure
    pub fn sub_points(&self, i: usize) -> u16 {
        self.subs[i].count()
    }
    /// Get the current sub-figure
    fn sub_current(&mut self) -> &mut SubFig {
        let len = self.subs.len();
        &mut self.subs[len - 1]
    }
    /// Add a new sub-figure
    fn sub_add(&mut self) {
        let vid = self.points.len() as u16;
        self.subs.push(SubFig::new(vid));
    }
    /// Add a point to the current sub-figure
    fn sub_add_point(&mut self) {
        let mut sub = self.sub_current();
        sub.n_points += 1;
    }
    /// Get the sub-figure at a specified vertex ID
    fn sub_at(&self, vid: u16) -> &SubFig {
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
    pub fn next(&self, vid: u16, dir: FigDir) -> u16 {
        let sub = self.sub_at(vid);
        sub.next(vid, dir)
    }
    /// Get the next vertex with a different Y
    fn next_y(&self, vid: u16, dir: FigDir) -> u16 {
        let py = self.points[vid as usize].y;
        let sub = self.sub_at(vid);
        let mut v = sub.next(vid, dir);
        while v != vid {
            let y = self.points[v as usize].y;
            if Fixed::cmp_f32(py, y) != Equal {
                return v;
            }
            v = sub.next(v, dir);
        }
        vid
    }
    /// Get the next vertex for an edge change
    fn next_edge(&self, vid: u16, dir: FigDir) -> u16 {
        let px = self.points[vid as usize].x;
        let py = self.points[vid as usize].y;
        let sub = self.sub_at(vid);
        let mut v = sub.next(vid, dir);
        while v != vid {
            let x = self.points[v as usize].x;
            let y = self.points[v as usize].y;
            if x < px || Fixed::cmp_f32(py, y) != Equal {
                return v;
            }
            v = sub.next(v, dir);
        }
        vid
    }
    /// Get the last vertex with the same Y
    fn same_y(&self, vid: u16, dir: FigDir) -> u16 {
        let py = self.points[vid as usize].y;
        let sub = self.sub_at(vid);
        let mut vp = vid;
        let mut v = sub.next(vid, dir);
        while v != vid {
            let y = self.points[v as usize].y;
            if Fixed::cmp_f32(py, y) != Equal {
                return vp;
            }
            vp = v;
            v = sub.next(v, dir);
        }
        vid
    }
    /// Add a point.
    ///
    /// * `pt` Point to add (z indicates stroke width).
    pub fn add_point(&mut self, pt: Vec3) {
        let n_pts = self.points.len();
        if n_pts < u16::max_value() as usize {
            if self.sub_current().done {
                self.sub_add();
            }
            self.points.push(pt);
            self.sub_add_point();
        }
    }
    /// Close the current sub-figure.
    ///
    /// * `joined` If true, join ends of sub-figure.
    pub fn close(&mut self, joined: bool) {
        if self.points.len() > 0 {
            let mut sub = self.sub_current();
            sub.joined = joined;
            sub.done = true;
        }
    }
    /// Reset the figure (clear all points).
    pub fn reset(&mut self) {
        self.points.clear();
        self.subs.clear();
        self.subs.push(SubFig::new(0u16));
    }
    /// Compare two figure vertex IDs
    fn compare_vids(&self, v0: u16, v1: u16) -> Ordering {
        let p0 = &self.points[v0 as usize];
        let p1 = &self.points[v1 as usize];
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
    /// * `scan_buf` Scan buffer (must be same width as mask, with height 1).
    /// * `rule` Fill rule.
    pub fn fill(&mut self, mask: &mut Mask, scan_buf: &mut Mask, rule: FillRule)
    {
        let n_points = self.points.len() as u16;
        let mut vids: Vec<u16> = (0u16..n_points).collect();
        vids.sort_by(|a,b| self.compare_vids(*a, *b));
        let mut scan = Scanner::new(self, mask, scan_buf, rule);
        for vid in vids {
            if scan.is_complete() {
                break;
            }
            scan.scan_vertex(vid);
        }
    }
    /// Get boundary of stroke between two points
    pub fn stroke_boundary(&self, v0: u16, v1: u16) -> (Vec2, Vec2) {
        let p0 = self.points[v0 as usize];
        let p1 = self.points[v1 as usize];
        let pp0 = Vec2::new(p0.x, p0.y);
        let pp1 = Vec2::new(p1.x, p1.y);
        let vr = (pp0 - pp1).left().normalize();
        let pr0 = pp0 + vr * (p0.z / 2f32);
        let pr1 = pp1 + vr * (p1.z / 2f32);
        (pr0, pr1)
    }
}

impl<'a> Scanner<'a> {
    /// Create a new figure scanner struct
    fn new(fig: &'a mut Fig, mask: &'a mut Mask, scan_buf: &'a mut Mask,
           rule: FillRule) -> Scanner<'a>
    {
        assert!(scan_buf.height() == 1);
        assert!(mask.width() == scan_buf.width());
        let y_bot = Fixed::from_i32(mask.height() as i32);
        let edges = Vec::with_capacity(16);
        Scanner {
            fig:      fig,
            mask:     mask,
            scan_buf: scan_buf,
            edges:    edges,
            rule:     rule,
            n_fill:   0,
            y_now:    FX_ZERO,
            y_prev:   FX_ZERO,
            y_bot:    y_bot,
        }
    }
    /// Get the scan line for a Y value
    fn line(y: Fixed) -> i32 {
        (y - FX_EPSILON).to_i32()
    }
    /// Scan figure to a given vertex
    fn scan_vertex(&mut self, vid: u16) {
        let p = self.fig.points[vid as usize];
        let y_vtx = Fixed::from_f32(p.y);
        if self.edges.len() > 0 {
            self.scan_to_y(y_vtx);
        } else {
            self.y_now = y_vtx;
            self.y_prev = y_vtx;
        }
        self.update_edges(vid);
    }
    /// Scan figure, rasterizing all lines above a vertex
    fn scan_to_y(&mut self, y_vtx: Fixed) {
        while self.y_now < y_vtx && !self.is_complete() {
            self.y_prev = self.y_now;
            self.y_now = cmp::min(y_vtx, self.y_now.floor() + FX_ONE);
            if self.is_next_line() {
                self.advance_edges();
            }
            self.calculate_x_limits();
            if self.y_now > FX_ZERO {
                self.scan_once();
                self.accumulate_mask();
            }
        }
    }
    /// Check if scan is complete (reached bottom of mask)
    fn is_complete(&self) -> bool {
	    Scanner::line(self.y_now) >= Scanner::line(self.y_bot)
    }
    /// Check if scan has advanced to the next line
    fn is_next_line(&self) -> bool {
        Scanner::line(self.y_now) > Scanner::line(self.y_prev)
    }
    /// Advance all edges to the next line
    fn advance_edges(&mut self) {
        for e in self.edges.iter_mut() {
            e.x_bot = e.x_bot + e.islope;
        }
    }
    /// Calculate the X limits on the current scan line for all edges
    fn calculate_x_limits(&mut self) {
        let part = (self.y_now - self.y_prev) < FX_ONE;
        for e in self.edges.iter_mut() {
            if part {
                let y_bot = self.y_now.ceil();
                let ypb = self.y_prev - y_bot;
                let xt = ypb * e.islope + e.x_bot;
                let ynb = self.y_now - y_bot;
                let xb = ynb * e.islope + e.x_bot;
                e.min_x = cmp::min(xt, xb);
                e.max_x = cmp::max(xt, xb);
            } else {
                let xt = e.x_bot - e.islope;
                e.min_x = cmp::min(xt, e.x_bot);
                e.max_x = cmp::max(xt, e.x_bot);
            }
            e.min_pix = e.min_x.to_i32();
            e.max_pix = e.max_x.to_i32();
        }
    }
    /// Scan once across all edges
    fn scan_once(&mut self) {
    	let mut e: Option<usize> = None;
	    self.sort_edges();
        self.n_fill = 0;
        self.scan_buf.clear();
        let n_edges = self.edges.len();
        for i in 0..n_edges {
            match (e, self.edge_fill(i)) {
                (Some(lt), false) => {
                    self.rasterize(lt, i);
                    e = None;
                },
                (None, true) => {
                    e = Some(i);
                },
                _ => {},
            };
        }
    }
    /// Sort edges by X at the bottom of the current scan line.
    fn sort_edges(&mut self) {
        self.edges.sort_by(|a,b| a.x_mid().v.cmp(&b.x_mid().v));
    }
    /// Check if the given edge starts a fill
    fn edge_fill(&mut self, i: usize) -> bool {
        let e = &self.edges[i];
        self.n_fill += match e.dir {
            FigDir::Forward => 1,
            FigDir::Reverse => -1,
        };
        match self.rule {
            FillRule::NonZero => self.n_fill != 0,
            FillRule::EvenOdd => (self.n_fill & 1) != 0,
        }
    }
    /// Rasterize the current scan line between 2 edges
    fn rasterize(&mut self, lt: usize, rt: usize) {
        let left = &self.edges[lt];
        let right = &self.edges[rt];
        let w = self.mask.width() as i32;
        let max_pix = cmp::min(right.max_pix, w - 1);
        let min_rt = cmp::min(max_pix, right.min_pix);
        assert!(self.y_now > self.y_prev);
        let cov = self.y_now - self.y_prev;
        let mut cov_pix_l = FX_ZERO;
        let mut cov_pix_r = FX_ZERO;
        let mut x = cmp::max(left.min_pix, 0);
        while x <= max_pix {
            if x > left.max_pix && x < min_rt {
                self.scan_buf.fill(x as usize, (min_rt - x) as usize,
                    pixel_cov(cov) as u8);
                x = min_rt;
                continue;
            }
            cov_pix_l = left.scan_pix(x, cov_pix_l);
            cov_pix_r = right.scan_pix(x, cov_pix_r);
            let sam = pixel_cov(cov * (cov_pix_l - cov_pix_r));
            self.scan_buf.set(x, sam);
            x += 1;
        }
    }
    /// Accumulate scan buffer over mask
    fn accumulate_mask(&mut self) {
        assert!(self.y_now > FX_ZERO);
        let y = Scanner::line(self.y_now) as u32;
        self.mask.accumulate(&self.scan_buf, y);
    }
    /// Update edges at a given vertex
    fn update_edges(&mut self, vid: u16) {
        let vp = self.fig.next_edge(vid, FigDir::Reverse);
        let vn = self.fig.next_edge(vid, FigDir::Forward);
        if (vp != vid) && (vn != vid) {
            let y = self.fig.points[vid as usize].y;
            let cp = Fixed::cmp_f32(self.fig.points[vp as usize].y, y);
            let cn = Fixed::cmp_f32(self.fig.points[vn as usize].y, y);
            match (cp, cn) {
                (Less,    Less)    => self.edge_merge(vid),
                (Greater, Greater) => self.edge_split(vp, vn),
                _                  => self.edge_regular(vid),
            }
        }
    }
    /// Remove two edges at a merge vertex
    fn edge_merge(&mut self, vid: u16) {
        let fig = &self.fig;
        let mut i = self.edges.len();
        while i > 0 {
            i -= 1;
            let vtx = self.edges[i].vtx;
            if (fig.same_y(vtx, FigDir::Forward) == vid) ||
               (fig.same_y(vtx, FigDir::Reverse) == vid)
            {
                self.edges.remove(i);
            }
        }
    }
    /// Add two edges at a split vertex
    fn edge_split(&mut self, v0: u16, v1: u16) {
        let fig = &self.fig;
        let v0u = fig.next(v0, FigDir::Forward);    // Find upper vtx of edge 0
        let p0u = &fig.points[v0u as usize];        // Upper point of edge 0
        let p0 = &fig.points[v0 as usize];          // Lower point of edge 0
        self.edges.push(Edge::new(v0u, v0, p0u, p0, FigDir::Reverse));
        let v1u = fig.next(v1, FigDir::Reverse);    // Find upper vtx of edge 1
        let p1u = &fig.points[v1u as usize];        // Upper point of edge 1
        let p1 = &fig.points[v1 as usize];          // Lower point of edge 1
        self.edges.push(Edge::new(v1u, v1, p1u, p1, FigDir::Forward));
    }
    /// Update one edge at a regular vertex
    fn edge_regular(&mut self, vid: u16) {
        let fig = &self.fig;
        for i in 0..self.edges.len() {
            if vid == self.edges[i].vtx {
                let dir = self.edges[i].dir;
                let vn = fig.next_y(vid, dir);          // Find lower vertex
                let v = fig.next_y(vn, opposite(dir));  // Find upper vertex
                let p = &fig.points[v as usize];
                let pn = &fig.points[vn as usize];
                self.edges[i] = Edge::new(v, vn, p, pn, dir);
                break;
            }
        }
    }
}

/// Calculate pixel coverage
///
/// fcov Total coverage (0 to 1 fixed-point).
/// return Total pixel coverage (0 to 255).
fn pixel_cov(fcov: Fixed) -> i32 {
    // Round to nearest cov value
    let n = (fcov.v + (1 << 7)) >> 8;
    cmp::min(cmp::max(n, 0), 255)
}

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
