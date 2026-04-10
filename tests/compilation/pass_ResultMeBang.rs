#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2::Try;

#[derive(Try)]
#[must_use]
enum Eightball<T> {
    Yes(T),
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

impl<T> std::ops::FromResidual<Result<std::convert::Infallible, Eightball<!>>> for Eightball<T> {
    fn from_residual(residual: Result<std::convert::Infallible, Eightball<!>>) -> Self {
        match residual {
            Result::Err(e) => match e {
                Eightball::No => Self::No
            }
        }
    }
}

fn main() {}
