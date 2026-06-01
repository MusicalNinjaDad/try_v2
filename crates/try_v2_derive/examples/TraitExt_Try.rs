#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]
#![allow(clippy::disallowed_names)]

use log::info;
use std::{fmt::Display, ops::Add};

use try_v2_derive::Try;

use Counter::Count;

#[derive(Debug, Try)]
#[must_use]
enum Counter<N> {
    Count(N),
    Uninitialised,
}

impl Counter<i32> {
    fn new() -> Self {
        info!("new counter initialised");
        Count(0)
    }
}

// More versatile implementation, better separating responsibilities:
// implementation details are owned here, type specifics at usage site.
impl<N, M> Add<M> for Counter<N>
where
    N: Add<M, Output = N> + Display,
    M: Display,
{
    type Output = Self;

    fn add(self, rhs: M) -> Self::Output {
        info!("adding {rhs} to counter");
        let n = self? + rhs;
        info!("new value {n}");
        Self::Count(n)
    }
}

fn main() {
    let foo = Counter::new();
    assert!(matches!(foo + 2, Count(n) if n ==2));
}
