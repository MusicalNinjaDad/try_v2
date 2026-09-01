#![cfg_attr(unstable_never_type, feature(never_type))]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2_derive::{FromResidual, Try};

#[derive(Debug, Try, FromResidual, PartialEq)]
#[FromResidual(Option<Self>)]
#[must_use]
enum EightBall<Y, N> {
    Yes(Y),
    RollAgain,
    No(N),
}

#[allow(dead_code)]
fn maybe_eightball() -> Option<EightBall<(), ()>> {
    let _ = EightBall::RollAgain?;
    let _ = Some(EightBall::RollAgain)??;
    None
}

#[test]
fn test_maybe_eightball() {
    let ret = maybe_eightball();
    assert_eq!(ret, Some(EightBall::RollAgain));
}

fn main() {}
