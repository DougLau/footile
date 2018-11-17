// fig.rs    A 2D rasterizer.
//
// Copyright (c) 2017-2018  Douglas P Lau
//
use std::cmp;
use std::cmp::Ordering;
use std::cmp::Ordering::*;
use std::fmt;
use geom::Vec2;
use mask::Mask;
use path::FillRule;
use fixed::Fixed;

/// Vertex ID
type Vid = u16;

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

/// Compare two f32 for fixed-point equality
fn cmp_fixed(a: f32, b: f32) -> Ordering {
    Fixed::from(a).cmp(&Fixed::from(b))
}

/// Get the line of a value
fn line_of(f: Fixed) -> i32 {
    (f - Fixed::EPSILON).into()
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
        debug_assert!(v0 != v1);
        let dx = Fixed::from(p1.x - p0.x);  // delta X
        let dy = Fixed::from(p1.y - p0.y);  // delta Y
        debug_assert!(dy > Fixed::ZERO);
        let step_pix = Edge::calculate_step(dx, dy);
        let islope = dx / dy;
        let y0 = Fixed::from(p0.y);
        let y1 = Fixed::from(p1.y);
        let y0f = if y0.fract() > Fixed::ZERO { Some(y0.into()) } else { None };
        let y1f = if y1.fract() > Fixed::ZERO { Some(y1.into()) } else { None };
        let fm = (y0.ceil() - y0) * islope;
        let x_bot = fm + Fixed::from(p0.x);
        Edge {
            v1       : v1,
            y0f      : y0f,
            y1f      : y1f,
            dir      : dir,
            step_pix : step_pix,
            islope   : islope,
            x_bot    : x_bot,
            min_x    : Fixed::ZERO,
            max_x    : Fixed::ZERO,
        }
    }
    /// Calculate the step for each pixel on an edge
    fn calculate_step(dx: Fixed, dy: Fixed) -> Fixed {
        if dx != Fixed::ZERO {
            cmp::min(Fixed::ONE, (dy / dx).abs())
        } else {
            Fixed::ZERO
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
        self.min_x.into()
    }
    /// Get the maximum X pixel
    fn max_pix(&self) -> i32 {
        self.max_x.into()
    }
    /// Get coverage of first pixel on edge.
    fn first_cov(&self) -> Fixed {
        let r = if self.min_pix() == self.max_pix() {
            (Fixed::ONE - self.x_mid().fract())
        } else {
            (Fixed::ONE - self.min_x.fract()) * Fixed::HALF
        };
        self.step_cov(r)
    }
    /// Get pixel coverage.
    fn step_cov(&self, r: Fixed) -> Fixed {
        if self.step_pix > Fixed::ZERO {
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
        let s_n = pixel_cov(self.step_cov(Fixed::ONE));
        debug_assert!(s_n > 0);
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
    /// Check if current sub-figure is done.
    fn sub_is_done(&self) -> bool {
        self.subs.last().unwrap().done
    }
    /// Mark sub-figure done.
    fn sub_set_done(&mut self) {
        let start = { self.sub_current().start };
        let pt = self.get_point(start);
        let c = self.coincident(pt);
        if c { self.points.pop(); }
        let sub = self.sub_current();
        debug_assert!(sub.n_points > 0);
        sub.done = true;
        if c { sub.n_points -= 1; }
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
            if cmp_fixed(py, y) != Equal {
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
            if p.x < pp.x || cmp_fixed(pp.y, p.y) != Equal {
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
            if cmp_fixed(py, y) != Equal {
                return vp;
            }
            vp = v;
            v = sub.next(v, dir);
        }
        vid
    }
    /// Get direction from top vertex.
    fn get_dir(&self, vid: Vid) -> FigDir {
        let p = self.get_point(vid);
        let p0 = self.get_point(self.next(vid, FigDir::Forward));
        let p1 = self.get_point(self.next(vid, FigDir::Reverse));
        if (p1 - p).widdershins(p0 - p) {
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
            let done = self.sub_is_done();
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
    ///
    /// NOTE: This must be called before filling in order to handle coincident
    ///       start/end points.
    pub fn close(&mut self) {
        if self.points.len() > 0 {
            self.sub_set_done();
        }
    }
    /// Compare two figure vertex IDs
    fn compare_vids(&self, v0: Vid, v1: Vid) -> Ordering {
        let p0 = self.get_point(v0);
        let p1 = self.get_point(v1);
        match cmp_fixed(p0.y, p1.y) {
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
        if n_points > 0 {
            debug_assert!(self.sub_is_done());
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
}

impl<'a> Scanner<'a> {
    /// Create a new figure scanner struct
    fn new(fig: &'a Fig, mask: &'a mut Mask, sgn_area: &'a mut [i16],
           dir: FigDir, rule: FillRule) -> Scanner<'a>
    {
        assert!(mask.width() <= sgn_area.len() as u32);
        let y_bot = Fixed::from(mask.height() as i32);
        let edges = Vec::with_capacity(16);
        Scanner {
            fig      : fig,
            mask     : mask,
            sgn_area : sgn_area,
            edges    : edges,
            dir      : dir,
            rule     : rule,
            y_now    : Fixed::ZERO,
            y_prev   : Fixed::ZERO,
            y_bot    : y_bot,
        }
    }
    /// Scan figure to a given vertex
    fn scan_vertex(&mut self, vid: Vid) {
        let y = self.get_y(vid);
        let y_vtx = Fixed::from(y);
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
            self.y_now = cmp::min(y_vtx, self.y_now.floor() + Fixed::ONE);
            if self.is_next_line() {
                self.advance_edges();
            }
            if self.y_now > Fixed::ZERO {
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
        line_of(self.y_now) >= line_of(self.y_bot)
    }
    /// Check if scan is at bottom of line
    fn is_line_bottom(&self) -> bool {
        self.y_now.fract() == Fixed::ZERO
    }
    /// Check if scan has advanced to the next line
    fn is_next_line(&self) -> bool {
        line_of(self.y_now) > line_of(self.y_prev)
    }
    /// Advance all edges to the next line
    fn advance_edges(&mut self) {
        for e in self.edges.iter_mut() {
            e.x_bot = e.x_bot + e.islope;
        }
    }
    /// Check if current scan line is partial
    fn is_partial(&self) -> bool {
        (self.y_now - self.y_prev) < Fixed::ONE
    }
    /// Scan partial edges
    fn scan_partial(&mut self) {
        let cov_full = self.scan_coverage();
        debug_assert!(cov_full <= 256);
        if cov_full <= 0 {
            return;
        }
        let y = line_of(self.y_now);
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
        let y = line_of(self.y_now);
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
        if self.y_now > Fixed::ZERO && self.y_now <= self.y_bot {
            let y = line_of(self.y_now) as u32;
            self.mask.scan_accumulate(self.sgn_area, y, self.rule);
        }
    }
    /// Get full scan coverage
    fn scan_coverage(&self) -> i16 {
        debug_assert!(self.y_now > self.y_prev);
        debug_assert!(self.y_now <= self.y_prev + Fixed::ONE);
        let scan_now = pixel_cov(self.y_now.fract());
        let scan_prev = pixel_cov(self.y_prev.fract());
        if scan_now == scan_prev && self.y_now.fract() > Fixed::ZERO {
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
            let cp = cmp_fixed(self.get_y(vp), y);
            let cn = cmp_fixed(self.get_y(vn), y);
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
    debug_assert!(fcov >= Fixed::ZERO && fcov <= Fixed::ONE);
    // Round to nearest cov value
    let n: i32 = (fcov << 8).round().into();
    n as i16
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn compare_fixed() {
        assert!(cmp_fixed(0f32, 0f32) == Ordering::Equal);
        assert!(cmp_fixed(0f32, 0.00001f32) == Ordering::Equal);
        assert!(cmp_fixed(0f32, 0.0001f32) == Ordering::Less);
        assert!(cmp_fixed(0f32, -0.0001f32) == Ordering::Greater);
    }
    #[test]
    fn fig_3x3() {
        let mut m = Mask::new(3, 3);
        let mut s = vec!(0; 3);
        let mut f = Fig::new();
        f.add_point(Vec2::new(0f32, 0f32));
        f.add_point(Vec2::new(3f32, 3f32));
        f.add_point(Vec2::new(0f32, 3f32));
        f.close();
        f.fill(&mut m, &mut s, FillRule::NonZero);
        let p: Vec<_> = m.pixels().iter().cloned().collect();
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
        f.close();
        f.fill(&mut m, &mut s, FillRule::NonZero);
        let p: Vec<_> = m.pixels().iter().cloned().collect();
        assert!(p == [242, 214, 186, 158, 130, 102, 74, 46, 18]);
    }
}
