#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2::Try;

#[derive(Try)]
#[must_use]
enum Eightball<Y> {
    Yes(Y),
    No,
}

struct Even(i32);

impl TryFrom<i32> for Even {
    type Error = Eightball<!>;

    fn try_from(num: i32) -> Result<Even, Eightball<!>> {
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

impl<Y, E> std::ops::FromResidual<Result<std::convert::Infallible, E>> for Eightball<Y>
where
    E: Into<Eightball<!>>,
{
    fn from_residual(residual: Result<std::convert::Infallible, E>) -> Self {
        match residual {
            Result::Err(e) => {
                let bang: Eightball<!> = e.into();
                Self::from_residual(bang)
            }
        }
    }
}

fn main() {}
