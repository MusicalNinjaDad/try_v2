#![allow(unused)]
#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use std::ops::Try;

use try_v2_derive::{IntoIterator, Try};

#[derive(Debug, Try, IntoIterator)]
#[must_use]
enum EightBall<Y, N> {
    Yes(Y),
    RollAgain,
    No(N),
}

#[test]
fn into() {
    assert_eq!(Some(5), EightBall::<i32, i32>::Yes(5).into_iter().next());
    assert_eq!(None, EightBall::<i32, i32>::RollAgain.into_iter().next());
    assert_eq!(None, EightBall::<i32, i32>::No(5).into_iter().next());
}

fn main() {}
