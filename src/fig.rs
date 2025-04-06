// fig.rs    A 2D rasterizer.
//
// Copyright (c) 2017-2025  Douglas P Lau
//
use crate::fixed::Fixed;
use crate::imgbuf::{matte_src_over_even_odd, matte_src_over_non_zero};
use crate::path::FillRule;
use crate::vid::Vid;
use pix::chan::{Ch8, Linear, Premultiplied};
use pix::el::Pixel;
use pix::matte::Matte8;
use pix::ops::SrcOver;
use pix::{Raster, RowsMut};
use pointy::Pt;
use std::any::TypeId;
use std::cmp::Ordering;
use std::cmp::Ordering::*;
use std::fmt;
use std::ops::Sub;

/// A 2D point with fixed-point values
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct FxPt {
    x: Fixed,
    y: Fixed,
}

/// Figure direction enum
#[derive(Clone, Copy, Debug, PartialEq)]
enum FigDir {
    Forward,
    Reverse,
}

/// Sub-figure structure
struct SubFig {
    /// Starting point
    start: Vid,
    /// Number of points
    n_points: usize,
    /// Done flag
    done: bool,
}

/// Edge structure
#[derive(Debug)]
struct Edge {
    /// Lower vertex ID
    v1: Vid,
    /// Upper vertex Y
    y_upper: Fixed,
    /// Lower vertex Y
    y_lower: Fixed,
    /// Figure direction from upper to lower
    dir: FigDir,
    /// Change in cov per pix on current row
    step_pix: Fixed,
    /// Inverse slope (delta_x / delta_y)
    inv_slope: Fixed,
    /// X at bottom of current row
    x_bot: Fixed,
    /// Minimum X on current row
    min_x: Fixed,
    /// Maximum X on current row
    max_x: Fixed,
}

/// A Fig is a series of 2D points which can be rendered to an image raster.
pub struct Fig {
    /// All pionts
    points: Vec<FxPt>,
    /// All sub-figures
    subs: Vec<SubFig>,
}

/// Figure scanner structure
struct Scanner<'a, P>
where
    P: Pixel<Chan = Ch8, Alpha = Premultiplied, Gamma = Linear>,
{
    /// The figure
    fig: &'a Fig,
    /// Fill rule
    rule: FillRule,
    /// Figure direction
    dir: FigDir,
    /// Destination raster rows
    rows: RowsMut<'a, P>,
    /// Color to fill
    clr: P,
    /// Signed area buffer
    sgn_area: &'a mut [i16],
    /// Active edges
    edges: Vec<Edge>,
}

impl Sub for FxPt {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        FxPt::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl FxPt {
    /// Create a new point.
    fn new(x: Fixed, y: Fixed) -> Self {
        FxPt { x, y }
    }

    /// Calculate winding order for two vectors.
    ///
    /// The vectors should be initialized as edges pointing toward the same
    /// point.
    /// Returns true if the winding order is widdershins (counter-clockwise).
    fn widdershins(self, rhs: Self) -> bool {
        // Cross product (with Z zero) is used to determine the winding order.
        (self.x * rhs.y) > (rhs.x * self.y)
    }
}

impl FigDir {
    /// Get the opposite direction
    fn opposite(self) -> Self {
        match self {
            FigDir::Forward => FigDir::Reverse,
            FigDir::Reverse => FigDir::Forward,
        }
    }
}

impl SubFig {
    /// Create a new sub-figure
    fn new(start: Vid) -> SubFig {
        SubFig {
            start,
            n_points: 0,
            done: false,
        }
    }

    /// Get next vertex within a sub-figure
    fn next(&self, vid: Vid, dir: FigDir) -> Vid {
        match dir {
            FigDir::Forward => {
                let v = vid + 1;
                if v < self.start + self.n_points {
                    v
                } else {
                    self.start
                }
            }
            FigDir::Reverse => {
                if vid > self.start {
                    vid - 1
                } else if self.n_points > 0 {
                    self.start + self.n_points - 1
                } else {
                    self.start
                }
            }
        }
    }
}

/// Get the row of a Y value
fn row_of(y: Fixed) -> i32 {
    y.into()
}

impl Edge {
    /// Create a new edge
    ///
    /// * `v0` Upper vertex.
    /// * `v1` Lower vertex.
    /// * `p0` Upper point.
    /// * `p1` Lower point.
    /// * `dir` Direction from upper to lower vertex.
    fn new(v0: Vid, v1: Vid, p0: FxPt, p1: FxPt, dir: FigDir) -> Edge {
        debug_assert_ne!(v0, v1);
        let delta_x = p1.x - p0.x;
        let delta_y = p1.y - p0.y;
        debug_assert!(delta_y > Fixed::ZERO);
        let step_pix = Edge::calculate_step(delta_x, delta_y);
        let inv_slope = delta_x / delta_y;
        let y_upper = p0.y;
        let y_lower = p1.y;
        let y_bot = (y_upper + Fixed::ONE).floor() - y_upper;
        let x_bot = p0.x + inv_slope * y_bot;
        Edge {
            v1,
            y_upper,
            y_lower,
            dir,
            step_pix,
            inv_slope,
            x_bot,
            min_x: Fixed::ZERO,
            max_x: Fixed::ZERO,
        }
    }

    /// Calculate the step for each pixel on an edge
    fn calculate_step(delta_x: Fixed, delta_y: Fixed) -> Fixed {
        if delta_x != Fixed::ZERO {
            (delta_y / delta_x).abs().min(Fixed::ONE)
        } else {
            Fixed::ZERO
        }
    }

    /// Get the minimum X pixel
    fn min_pix(&self) -> i32 {
        self.min_x.into()
    }

    /// Get the maximum X pixel
    fn max_pix(&self) -> i32 {
        self.max_x.into()
    }

    /// Get the X midpoint for the current row.
    fn mid_x(&self) -> Fixed {
        self.max_x.avg(self.min_x)
    }

    /// Check for the edge starting row.
    fn is_starting(&self, y_row: i32) -> bool {
        row_of(self.y_upper) == y_row
    }

    /// Check for the edge ending row.
    fn is_ending(&self, y_row: i32) -> bool {
        row_of(self.y_lower) == y_row
    }

    /// Get pixel coverage of starting row.
    fn starting_cov(&self) -> i16 {
        let y_row = row_of(self.y_upper);
        self.continuing_cov(y_row) - pixel_cov(self.y_upper.fract())
    }

    /// Calculate X limits for the starting row.
    fn calculate_x_limits_starting(&mut self) {
        let y_row = row_of(self.y_upper);
        let y0 = Fixed::ONE - self.y_upper.fract();
        let x0 = self.x_bot - self.inv_slope * y0;
        self.set_x_limits(x0, y_row);
    }

    /// Get pixel coverage of continuing row.
    fn continuing_cov(&self, y_row: i32) -> i16 {
        debug_assert!(y_row <= row_of(self.y_lower));
        if self.is_ending(y_row) {
            pixel_cov(self.y_lower.fract())
        } else {
            256
        }
    }

    /// Calculate X limits for a continuing row.
    fn calculate_x_limits_continuing(&mut self, y_row: i32) {
        debug_assert!(!self.is_starting(y_row));
        let x0 = self.x_bot - self.inv_slope;
        self.set_x_limits(x0, y_row);
    }

    /// Set X limits
    fn set_x_limits(&mut self, x0: Fixed, y_row: i32) {
        let x1 = if self.is_ending(y_row) {
            let y1 = self.y_lower.ceil() - self.y_lower;
            self.x_bot - self.inv_slope * y1
        } else {
            self.x_bot
        };
        self.min_x = x0.min(x1);
        self.max_x = x0.max(x1);
    }

    /// Scan signed area of current row.
    ///
    /// * `dir` Direction of edge.
    /// * `cov` Pixel coverage of current row (1 - 256).
    /// * `area` Signed area buffer.
    fn scan_area(&self, dir: FigDir, cov: i16, area: &mut [i16]) {
        let ed = if self.dir == dir { 1 } else { -1 };
        let full_cov = Fixed::from(cov as f32 / 256.0);
        let mut x_cov = self.first_cov(full_cov); // total coverage at X
        let step_cov = self.step_cov(Fixed::ONE); // coverage change per step
        debug_assert!(step_cov > Fixed::ZERO);
        let mut sum_pix = 0; // cumulative sum of pixel coverage
        for x in self.min_pix()..area.len() as i32 {
            let x_pix = pixel_cov(x_cov).min(cov);
            let p = x_pix - sum_pix; // pixel coverage at X
            area[x.max(0) as usize] += p * ed;
            sum_pix += p;
            if sum_pix >= cov {
                break;
            }
            x_cov = (x_cov + step_cov).min(Fixed::ONE);
        }
    }

    /// Get coverage of first pixel on edge.
    fn first_cov(&self, full_cov: Fixed) -> Fixed {
        let r = if self.min_pix() == self.max_pix() {
            (Fixed::ONE - self.mid_x().fract()) * full_cov
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
}

impl fmt::Debug for Fig {
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

impl Fig {
    /// Create a figure rasterizer
    pub fn new() -> Fig {
        let points = Vec::with_capacity(1024);
        let mut subs = Vec::with_capacity(16);
        subs.push(SubFig::new(Vid(0)));
        Fig { points, subs }
    }

    /// Get the current sub-figure
    fn sub_current(&self) -> &SubFig {
        self.subs.last().unwrap()
    }

    /// Get the current sub-figure mutably
    fn sub_current_mut(&mut self) -> &mut SubFig {
        self.subs.last_mut().unwrap()
    }

    /// Add a new sub-figure
    fn sub_add(&mut self) {
        let vid = Vid::from(self.points.len());
        self.subs.push(SubFig::new(vid));
    }

    /// Add a point to the current sub-figure
    fn sub_add_point(&mut self) {
        self.sub_current_mut().n_points += 1;
    }

    /// Check if current sub-figure is done.
    fn sub_is_done(&self) -> bool {
        self.subs.last().unwrap().done
    }

    /// Mark sub-figure done.
    fn sub_set_done(&mut self) {
        let sub = self.sub_current();
        if sub.n_points > 0 {
            let pt = self.point(sub.start);
            if self.is_coincident(pt) {
                self.points.pop();
                self.sub_current_mut().n_points -= 1;
            }
            self.sub_current_mut().done = true;
        }
    }

    /// Get the sub-figure at a specified vertex ID.
    fn sub_at(&self, vid: Vid) -> &SubFig {
        for sub in self.subs.iter() {
            if vid < sub.start + sub.n_points {
                return sub;
            }
        }
        // Invalid vid indicates bug
        unreachable!();
    }

    /// Get the next vertex.
    fn next(&self, vid: Vid, dir: FigDir) -> Vid {
        self.sub_at(vid).next(vid, dir)
    }

    /// Get direction from top-left vertex.
    fn get_dir(&self, vid: Vid) -> FigDir {
        let p = self.point(vid);
        let p0 = self.point(self.next(vid, FigDir::Forward));
        let p1 = self.point(self.next(vid, FigDir::Reverse));
        if (p1 - p).widdershins(p0 - p) {
            FigDir::Forward
        } else {
            FigDir::Reverse
        }
    }

    /// Get a point.
    ///
    /// * `vid` Vertex ID.
    fn point(&self, vid: Vid) -> FxPt {
        self.points[usize::from(vid)]
    }

    /// Get Y value at a vertex.
    fn get_y(&self, vid: Vid) -> Fixed {
        self.point(vid).y
    }

    /// Add a point.
    ///
    /// * `pt` Point to add.
    pub fn add_point<P: Into<Pt<f32>>>(&mut self, pt: P) {
        let n_pts = self.points.len();
        if n_pts < usize::from(Vid::MAX) {
            let done = self.sub_is_done();
            if done {
                self.sub_add();
            }
            let pt = pt.into();
            let pt = FxPt::new(Fixed::from(pt.x), Fixed::from(pt.y));
            if done || !self.is_coincident(pt) {
                self.points.push(pt);
                self.sub_add_point();
            }
        }
    }

    /// Check if a point is coincident with previous point.
    fn is_coincident(&self, pt: FxPt) -> bool {
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
        if !self.points.is_empty() {
            self.sub_set_done();
        }
    }

    /// Compare two figure vertex IDs
    fn compare_vids(&self, v0: Vid, v1: Vid) -> Ordering {
        let p0 = self.point(v0);
        let p1 = self.point(v1);
        match p0.y.cmp(&p1.y) {
            Less => Less,
            Greater => Greater,
            Equal => p0.x.cmp(&p1.x),
        }
    }

    /// Fill the figure to an image raster.
    ///
    /// * `rule` Fill rule.
    /// * `raster` Output raster.
    /// * `clr` Color to fill.
    /// * `sgn_area` Signed area buffer.
    pub fn fill<P>(
        &self,
        rule: FillRule,
        raster: &mut Raster<P>,
        clr: P,
        sgn_area: &mut [i16],
    ) where
        P: Pixel<Chan = Ch8, Alpha = Premultiplied, Gamma = Linear>,
    {
        assert!(raster.width() <= sgn_area.len() as u32);
        let n_points = self.points.len();
        if n_points > 0 {
            assert!(self.sub_is_done());
            let mut vids: Vec<Vid> = (0..n_points).map(Vid::from).collect();
            vids.sort_by(|a, b| self.compare_vids(*a, *b));
            let dir = self.get_dir(vids[0]);
            let top_row = row_of(self.point(vids[0]).y);
            let region = (0, top_row.max(0), raster.width(), raster.height());
            let rows = raster.rows_mut(region);
            let mut scan = Scanner::new(self, rule, dir, rows, clr, sgn_area);
            scan.scan_vertices(vids, top_row);
        }
    }
}

impl<'a, P> Scanner<'a, P>
where
    P: Pixel<Chan = Ch8, Alpha = Premultiplied, Gamma = Linear>,
{
    /// Create a new figure scanner.
    fn new(
        fig: &'a Fig,
        rule: FillRule,
        dir: FigDir,
        rows: RowsMut<'a, P>,
        clr: P,
        sgn_area: &'a mut [i16],
    ) -> Scanner<'a, P> {
        let edges = Vec::with_capacity(16);
        Scanner {
            fig,
            rule,
            dir,
            rows,
            clr,
            sgn_area,
            edges,
        }
    }

    /// Get Y value at a vertex.
    fn get_y(&self, vid: Vid) -> Fixed {
        self.fig.get_y(vid)
    }

    /// Scan all vertices in order.
    fn scan_vertices(&mut self, vids: Vec<Vid>, top_row: i32) {
        let mut vids = vids.iter().peekable();
        let mut y_row = top_row;
        while let Some(row_buf) = self.rows.next() {
            self.scan_continuing_edges(y_row);
            while let Some(vid) = vids.peek() {
                let y_vtx = self.get_y(**vid);
                if row_of(y_vtx) > y_row {
                    break;
                }
                let vid = *vids.next().unwrap();
                self.update_edges(vid, FigDir::Forward);
                self.update_edges(vid, FigDir::Reverse);
            }
            self.rasterize_row(row_buf);
            self.advance_edges();
            y_row += 1;
        }
    }

    /// Scan edges continuing on this row.
    fn scan_continuing_edges(&mut self, y_row: i32) {
        let area = &mut self.sgn_area;
        for e in self.edges.iter_mut() {
            let cov = e.continuing_cov(y_row);
            if cov > 0 {
                e.calculate_x_limits_continuing(y_row);
                e.scan_area(self.dir, cov, area);
            }
        }
    }

    /// Advance all edges to the next row.
    fn advance_edges(&mut self) {
        for e in self.edges.iter_mut() {
            e.x_bot = e.x_bot + e.inv_slope;
        }
    }

    /// Update edges at a given vertex.
    fn update_edges(&mut self, vid: Vid, dir: FigDir) {
        let v = self.fig.next(vid, dir);
        if v != vid {
            let y = self.get_y(vid);
            match self.get_y(v).cmp(&y) {
                Greater => self.add_edge(vid, v, dir),
                Less => self.remove_edge(vid, dir.opposite()),
                _ => (),
            }
        }
    }

    /// Add an edge.
    fn add_edge(&mut self, v0: Vid, v1: Vid, dir: FigDir) {
        let fig = &self.fig;
        let p0 = fig.point(v0); // Upper point
        let p1 = fig.point(v1); // Lower point
        let mut e = Edge::new(v0, v1, p0, p1, dir);
        let cov = e.starting_cov();
        if cov > 0 {
            e.calculate_x_limits_starting();
            e.scan_area(self.dir, cov, self.sgn_area);
        }
        self.edges.push(e);
    }

    /// Remove an edge.
    fn remove_edge(&mut self, v1: Vid, dir: FigDir) {
        if let Some(i) = self.find_edge(v1, dir) {
            self.edges.swap_remove(i);
        }
    }

    /// Find an active edge
    fn find_edge(&self, v1: Vid, dir: FigDir) -> Option<usize> {
        for (i, e) in self.edges.iter().enumerate() {
            if v1 == e.v1 && dir == e.dir {
                return Some(i);
            }
        }
        None
    }

    /// Rasterize the current row.
    /// Signed area is zeroed upon return.
    fn rasterize_row(&mut self, row_buf: &mut [P]) {
        match self.rule {
            FillRule::NonZero => self.scan_non_zero(row_buf),
            FillRule::EvenOdd => self.scan_even_odd(row_buf),
        }
    }

    /// Accumulate scan area with non-zero fill rule.
    fn scan_non_zero(&mut self, dst: &mut [P]) {
        let clr = self.clr;
        let sgn_area = &mut self.sgn_area;
        if TypeId::of::<P>() == TypeId::of::<Matte8>() {
            // FIXME: only if clr is Matte8::new(255)
            matte_src_over_non_zero(dst, sgn_area);
            return;
        }
        let mut sum = 0;
        for (d, s) in dst.iter_mut().zip(sgn_area.iter_mut()) {
            sum += *s;
            *s = 0;
            let alpha = Ch8::from(saturating_cast_i16_u8(sum));
            d.composite_channels_alpha(&clr, SrcOver, &alpha);
        }
    }

    /// Accumulate scan area with even-odd fill rule.
    fn scan_even_odd(&mut self, dst: &mut [P]) {
        let clr = self.clr;
        let sgn_area = &mut self.sgn_area;
        if TypeId::of::<P>() == TypeId::of::<Matte8>() {
            // FIXME: only if clr is Matte8::new(255)
            matte_src_over_even_odd(dst, sgn_area);
            return;
        }
        let mut sum = 0;
        for (d, s) in dst.iter_mut().zip(sgn_area.iter_mut()) {
            sum += *s;
            *s = 0;
            let v = sum & 0xFF;
            let odd = sum & 0x100;
            let c = (v - odd).abs();
            let alpha = Ch8::from(saturating_cast_i16_u8(c));
            d.composite_channels_alpha(&clr, SrcOver, &alpha);
        }
    }
}

/// Cast an i16 to a u8 with saturation
fn saturating_cast_i16_u8(v: i16) -> u8 {
    v.clamp(0, 255) as u8
}

/// Calculate pixel coverage
///
/// fcov Total coverage (0 to 1 fixed-point).
/// return Total pixel coverage (0 to 256).
fn pixel_cov(fcov: Fixed) -> i16 {
    debug_assert!(fcov >= Fixed::ZERO && fcov <= Fixed::ONE);
    // Round to nearest pixel cov value
    let n: i32 = (fcov << 8).round().into();
    n as i16
}

#[cfg(test)]
mod test {
    use super::*;
    use pix::Raster;
    use pix::matte::Matte8;
    use pix::rgb::Rgba8p;

    #[test]
    fn fixed_pt() {
        let a = FxPt::new(2.0.into(), 1.0.into());
        let b = FxPt::new(3.0.into(), 4.0.into());
        let c = FxPt::new(Fixed::from(-1.0), 1.0.into());
        assert_eq!(b - a, FxPt::new(1.0.into(), 3.0.into()));
        assert!(a.widdershins(b));
        assert!(!b.widdershins(a));
        assert!(b.widdershins(c));
    }

    #[test]
    fn fig_3x3() {
        let clr = Rgba8p::new(99, 99, 99, 255);
        let mut m = Raster::with_clear(3, 3);
        let mut s = vec![0; 3];
        let mut f = Fig::new();
        f.add_point((1.0, 2.0));
        f.add_point((1.0, 3.0));
        f.add_point((2.0, 3.0));
        f.add_point((2.0, 2.0));
        f.close();
        f.fill(FillRule::NonZero, &mut m, clr, &mut s);
        #[rustfmt::skip]
        let v = [
            Rgba8p::default(), Rgba8p::default(), Rgba8p::default(),
            Rgba8p::default(), Rgba8p::default(), Rgba8p::default(),
            Rgba8p::default(), Rgba8p::new(99, 99, 99, 255), Rgba8p::default(),
        ];
        assert_eq!(m.pixels(), &v);
    }

    #[test]
    fn fig_9x1() {
        let clr = Matte8::new(255);
        let mut m = Raster::<Matte8>::with_clear(9, 1);
        let mut s = vec![0; 16];
        let mut f = Fig::new();
        f.add_point((0.0, 0.0));
        f.add_point((9.0, 1.0));
        f.add_point((0.0, 1.0));
        f.close();
        f.fill(FillRule::NonZero, &mut m, clr, &mut s);
        assert_eq!([242, 213, 185, 156, 128, 100, 71, 43, 14], m.as_u8_slice());
    }

    #[test]
    fn fig_x_bounds() {
        let clr = Matte8::new(255);
        let mut m = Raster::<Matte8>::with_clear(3, 3);
        let mut s = vec![0; 4];
        let mut f = Fig::new();
        f.add_point((-1.0, 0.0));
        f.add_point((-1.0, 3.0));
        f.add_point((3.0, 1.5));
        f.close();
        f.fill(FillRule::NonZero, &mut m, clr, &mut s);
        assert_eq!([112, 16, 0, 255, 224, 32, 112, 16, 0], m.as_u8_slice());
    }

    #[test]
    fn fig_partial() {
        let clr = Matte8::new(255);
        let mut m = Raster::<Matte8>::with_clear(1, 3);
        let mut s = vec![0; 4];
        let mut f = Fig::new();
        f.add_point((0.5, 0.0));
        f.add_point((0.5, 1.5));
        f.add_point((1.0, 3.0));
        f.add_point((1.0, 0.0));
        f.close();
        f.fill(FillRule::NonZero, &mut m, clr, &mut s);
        assert_eq!([128, 117, 43], m.as_u8_slice());
    }

    #[test]
    fn fig_partial2() {
        let clr = Matte8::new(255);
        let mut m = Raster::<Matte8>::with_clear(3, 3);
        let mut s = vec![0; 3];
        let mut f = Fig::new();
        f.add_point((1.5, 0.0));
        f.add_point((1.5, 1.5));
        f.add_point((2.0, 3.0));
        f.add_point((3.0, 3.0));
        f.add_point((3.0, 0.0));
        f.close();
        f.fill(FillRule::NonZero, &mut m, clr, &mut s);
        assert_eq!([0, 128, 255, 0, 117, 255, 0, 43, 255], m.as_u8_slice());
    }

    #[test]
    fn fig_partial3() {
        let clr = Matte8::new(255);
        let mut m = Raster::<Matte8>::with_clear(9, 1);
        let mut s = vec![0; 16];
        let mut f = Fig::new();
        f.add_point((0.0, 0.0));
        f.add_point((0.0, 0.3));
        f.add_point((9.0, 0.0));
        f.close();
        f.fill(FillRule::NonZero, &mut m, clr, &mut s);
        assert_eq!([73, 64, 56, 47, 39, 30, 22, 13, 4], m.as_u8_slice());
    }
}
