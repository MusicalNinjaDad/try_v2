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

impl<Y, N, Derive_TryConvert_ResultE: Into<EightBall<!, N>>>
    ::std::ops::FromResidual<
        ::std::result::Result<::std::convert::Infallible, Derive_TryConvert_ResultE>,
    > for EightBall<Y, N>
{
    #[inline]
    #[track_caller]
    fn from_residual(
        residual: ::std::result::Result<::std::convert::Infallible, Derive_TryConvert_ResultE>,
    ) -> Self {
        let Err(e) = residual;
        Self::from_residual(e.into())
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
