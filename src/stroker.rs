// stroker.rs   A path stroker.
//
// Copyright (c) 2017-2018  Douglas P Lau
//
use std::fmt;
use geom::Vec2w;

/// Vertex ID
pub type Vid = u16;

/// Stroke direction enum
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Dir {
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
    points : Vec<Vec2w>,        // all points
    subs   : Vec<SubStroke>,    // all sub-strokes
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
    pub fn new() -> Stroke {
        let points = Vec::with_capacity(1024);
        let mut subs = Vec::with_capacity(16);
        subs.push(SubStroke::new(0 as Vid));
        Stroke { points, subs }
    }
    /// Get the count of sub-strokes
    pub fn sub_count(&self) -> usize {
        self.subs.len()
    }
    /// Get start of a sub-strokes
    pub fn sub_start(&self, i: usize) -> Vid {
        self.subs[i].start
    }
    /// Get end of a sub-strokes
    pub fn sub_end(&self, i: usize) -> Vid {
        let sub = &self.subs[i];
        sub.next(sub.start, Dir::Reverse)
    }
    /// Check if a sub-stroke is joined
    pub fn sub_joined(&self, i: usize) -> bool {
        self.subs[i].joined
    }
    /// Get the number of points in a sub-stroke
    pub fn sub_points(&self, i: usize) -> Vid {
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
    pub fn next(&self, vid: Vid, dir: Dir) -> Vid {
        let sub = self.sub_at(vid);
        sub.next(vid, dir)
    }
    /// Get a point.
    ///
    /// * `vid` Vertex ID.
    pub fn get_point(&self, vid: Vid) -> Vec2w {
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
}
