#![cfg_attr(unstable_never_type, feature(never_type))]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2_derive::Try;

#[derive(Try)]
#[FromResidual(Result<_, Self::Residual>)]
#[must_use]
enum Eightball<Y> {
    Yes(Y),
    No,
}

#[allow(unused)]
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

#[allow(unused)]
fn even_string(num: i32) -> Eightball<String> {
    let n = Even::try_from(num)?;
    let s = format!("{}", n.0);
    Eightball::Yes(s)
}

fn main() {}
