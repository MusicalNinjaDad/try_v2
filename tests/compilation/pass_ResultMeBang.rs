#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2::Try;

#[derive(Try)]
#[must_use]
enum Eightball<Y, N> {
    Yes(Y),
    No(N),
}

struct Even(i32);

impl TryFrom<i32> for Even {
    type Error = Eightball<!, &'static str>;

    fn try_from(num: i32) -> Result<Even, Eightball<!, &'static str>> {
        if num % 2 == 0 {
            Result::Ok(Even(num))
        } else {
            Result::Err(Eightball::No("odd"))
        }
    }
}

fn even_string(num: i32) -> Eightball<String, &'static str> {
    let n = Even::try_from(num)?;
    let s = format!("{}", n.0);
    Eightball::Yes(s)
}

impl<Y, N, E> std::ops::FromResidual<Result<std::convert::Infallible, E>> for Eightball<Y, N>
where
    E: Into<Eightball<Y, N>>,
{
    fn from_residual(residual: Result<std::convert::Infallible, E>) -> Self {
        match residual {
            Result::Err(e) => e.into(),
        }
    }
}

fn main() {}
