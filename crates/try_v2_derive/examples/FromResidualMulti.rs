#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2_derive::{FromResidual, Try};

#[derive(Debug, Try, FromResidual, PartialEq)]
// #[FromResidual(Result<Self, _>)]
// #[FromResidual(Option<Self>)]
#[FromResidual(Result<Self, _>, Option<Self>)]
#[must_use]
enum EightBall<Y, N> {
    Yes(Y),
    RollAgain,
    No(N),
}

#[allow(dead_code)]
fn good_eightball() -> Result<EightBall<(), ()>, ()> {
    let _ = EightBall::RollAgain?;
    Err(())
}

#[allow(dead_code)]
fn maybe_eightball() -> Option<EightBall<(), ()>> {
    let _ = EightBall::RollAgain?;
    None
}

#[test]
fn test_good_eightball() {
    let ret = good_eightball();
    assert_eq!(ret, Ok(EightBall::RollAgain));
}

#[test]
fn test_maybe_eightball() {
    let ret = maybe_eightball();
    assert_eq!(ret, Some(EightBall::RollAgain));
}

fn main() {}
