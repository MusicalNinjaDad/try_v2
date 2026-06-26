#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use std::ops::Residual;

use try_v2_derive::{FromResidual, Try};

#[derive(Debug, Try, PartialEq)]
// #[FromResidual(Result<_, Self::Residual>)]
#[must_use]
enum EightBall<Y, N> {
    Yes(Y),
    RollAgain,
    No(N),
}

impl<
    Y,
    N,
    Derive_TryConvert_ResultT,
    Derive_TryConvert_ResultE,
> ::std::ops::FromResidual<<EightBall<Y,N> as std::ops::Try>::Residual>
    for ::std::result::Result<Derive_TryConvert_ResultT, Derive_TryConvert_ResultE>
    where <EightBall<Y,N> as ::std::ops::Try>::Residual: Into<Derive_TryConvert_ResultE>,
{
    #[inline]
    #[track_caller]
    fn from_residual(residual: <EightBall<Y, N> as ::std::ops::Try>::Residual) -> Self {
        ::std::result::Result::Err(residual.into())
    }
}

impl<N> From<EightBall<!, N>> for Option<N> {
    fn from(residual: EightBall<!, N>) -> Self {
        match residual {
            EightBall::RollAgain => None,
            EightBall::No(n) => Some(n),
        }
    }
}

#[allow(dead_code)]
fn eightball_error() -> Result<(), Option<EightBall<!, ()>>> {
    let _ = EightBall::<_, ()>::RollAgain?;
    let _ = EightBall::<_, ()>::Yes(())?;
    Ok(())
}

#[test]
fn test_maybe_eightball() {
    let ret = maybe_eightball();
    assert_eq!(ret, Some(EightBall::RollAgain));
}

fn main() {}
