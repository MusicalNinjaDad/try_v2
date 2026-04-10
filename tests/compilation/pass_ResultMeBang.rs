#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2::Try;
use std::convert::Infallible;

#[derive(Try)]
#[must_use]
enum Eightball<T> {
    Yes(T),
    No,
}

struct Even(i32);

impl TryFrom<i32> for Even {
    type Error = Eightball<Infallible>;

    fn try_from(num: i32) -> Result<Even, Eightball<Infallible>> {
        if num % 2 == 0 {
            Result::Ok(Even(num))
        } else {
            Result::Err(Eightball::No)
        }
    }
}

fn even_string(num: i32) -> Eightball<String> {
    let n = Even::try_from(num)?;
    let s = format!("{}", n.0);
    Eightball::Yes(s)
}

impl<T, E> std::ops::FromResidual<Result<std::convert::Infallible, E>> for Eightball<T>
where
    E: Into<Eightball<T>>,
{
    fn from_residual(residual: Result<std::convert::Infallible, E>) -> Self {
        match residual {
            Result::Err(e) => e.into(),
        }
    }
}

impl<T> From<Eightball<Infallible>> for Eightball<T> {
    fn from(no: Eightball<Infallible>) -> Self {
        match no {
            Eightball::No => Self::No,
        }
    }
}

fn main() {}
