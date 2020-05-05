// vid.rs    Vertex ID
//
// Copyright (c) 2020  Douglas P Lau
//
use std::convert::TryFrom;
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// Vertex ID
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Vid(pub u16);

impl Vid {
    /// Minimum vertex ID
    pub const MIN: Self = Vid(u16::MIN);

    /// Maximum vertex ID
    pub const MAX: Self = Vid(u16::MAX);
}

impl From<usize> for Vid {
    fn from(v: usize) -> Self {
        Vid(u16::try_from(v).expect("Invalid vertex ID"))
    }
}

impl From<Vid> for usize {
    fn from(v: Vid) -> Self {
        usize::from(v.0)
    }
}

impl<R> Add<R> for Vid
where
    R: Into<Vid>,
{
    type Output = Self;

    fn add(self, rhs: R) -> Self {
        Vid(self.0 + rhs.into().0)
    }
}

impl<R> AddAssign<R> for Vid
where
    R: Into<Vid>,
{
    fn add_assign(&mut self, rhs: R) {
        self.0 = self.0 + rhs.into().0;
    }
}

impl<R> Sub<R> for Vid
where
    R: Into<Vid>,
{
    type Output = Self;

    fn sub(self, rhs: R) -> Self {
        Vid(self.0 - rhs.into().0)
    }
}

impl<R> SubAssign<R> for Vid
where
    R: Into<Vid>,
{
    fn sub_assign(&mut self, rhs: R) {
        self.0 = self.0 - rhs.into().0;
    }
}
