#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2_derive::{FromResidual, Try};

#[derive(Debug, Try, FromResidual, PartialEq)]
// #[FromResidual(Result<Option<Self>,E>)]
#[must_use]
enum EightBall<Y, N> {
    Yes(Y),
    RollAgain,
    No(N),
}

// This won't work for Poll as Poll's Try implementation is borken ;(

impl<Y, N> std::ops::FromResidual<EightBall<!, N>> for Option<EightBall<Y, N>> {
    fn from_residual(residual: EightBall<!, N>) -> Self {
        let eightball = std::ops::FromResidual::from_residual(residual);
        std::ops::Try::from_output(eightball)
    }
}

impl<Y, N, E> std::ops::FromResidual<EightBall<!, N>> for Result<Option<EightBall<Y, N>>, E> {
    fn from_residual(residual: EightBall<!, N>) -> Self {
        let eightball = std::ops::FromResidual::from_residual(residual);
        std::ops::Try::from_output(eightball)
    }
}

#[allow(dead_code)]
fn maybe_eightball() -> Result<Option<EightBall<(), ()>>, ()> {
    let _ = EightBall::RollAgain?;
    Err(())
}

#[test]
fn test_maybe_eightball() {
    let ret = maybe_eightball();
    assert_eq!(ret, Ok(Some(EightBall::RollAgain)));
}

fn main() {}
