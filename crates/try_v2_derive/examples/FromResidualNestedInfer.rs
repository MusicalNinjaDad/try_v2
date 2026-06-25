#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2_derive::{FromResidual, Try};

#[derive(Debug, Try, FromResidual, PartialEq)]
#[FromResidual(Result<Self, _>)]
#[must_use]
enum EightBall<Y, N> {
    Yes(Y),
    RollAgain,
    No(N),
}

#[allow(dead_code)]
fn maybe_eightball() -> Result<EightBall<(), ()>, ()> {
    let _ = EightBall::RollAgain?;
    let _ = Ok(EightBall::RollAgain)??;
    Err(())
}

#[test]
fn test_maybe_eightball() {
    let ret = maybe_eightball();
    assert_eq!(ret, Ok(EightBall::RollAgain));
}

fn main() {}

// // Recursive expansion of FromResidual macro
// // ==========================================
//
// impl<Y, N> std::ops::FromResidual<<EightBall<Y, N> as std::ops::Try>::Residual>
//     for Result<EightBall<Y, N>, _>
// {
//     #[inline]
//     #[track_caller]
//     fn from_residual(residual: <EightBall<Y, N> as std::ops::Try>::Residual) -> Self {
//         std::ops::Try::from_output(std::ops::FromResidual::from_residual(residual))
//     }
// }

// // Output of cargo expand
// // =======================
//
// impl<
//     Y,
//     N,
//     FromResidual_Generic_0,
// > std::ops::FromResidual<<EightBall<Y, N> as std::ops::Try>::Residual>
// for Result<EightBall<Y, N>, FromResidual_Generic_0> {
//     #[inline]
//     #[track_caller]
//     fn from_residual(residual: <EightBall<Y, N> as std::ops::Try>::Residual) -> Self {
//         std::ops::Try::from_output(std::ops::FromResidual::from_residual(residual))
//     }
// }
