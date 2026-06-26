#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2_derive::Try;

#[derive(Debug, Try, PartialEq)]
#[FromResidual(Result<_, Self::Residual>)]
#[must_use]
enum EightBall<Y, N> {
    Yes(Y),
    RollAgain,
    No(N),
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
fn eightball_error() -> Result<(), Option<()>> {
    let _ = EightBall::<_, ()>::RollAgain?;
    EightBall::<_, ()>::Yes(())?;
    Ok(())
}

#[test]
fn test_maybe_eightball() {
    let ret = eightball_error();
    assert_eq!(ret, Err(None));
}

fn main() {}
