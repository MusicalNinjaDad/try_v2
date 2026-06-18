#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2_derive::{Try, FromResidual};

#[derive(Debug, Try, FromResidual)]
#[FromResidual(Option<Self>)]
#[must_use]
enum EightBall<Y, N> {
    Yes(Y),
    RollAgain,
    No(N),
}

fn maybe_eightball() -> Option<EightBall<(),()>> {
    let _ = EightBall::RollAgain?;
    let _ = Some(EightBall::RollAgain)??;
    None
}



fn main() {}
