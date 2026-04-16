#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]
#![feature(associated_type_defaults)]

use try_v2::{Try, Try_ConvertResult};

#[derive(Try, Try_ConvertResult)]
#[must_use]
enum Eightball<Y> {
    Yes(Y),
    No,
}

struct Even(i32);

struct Even2(i32);

trait TryFrom2<T>: std::marker::Sized {
    type Error;
    type Return: std::ops::Try = Result<Self, Self::Error>;

    fn try_from2(value: T) -> Self::Return;
}

impl TryFrom2<i32> for Even {
    type Error = ();
    type Return = Eightball<Self>;

    fn try_from2(num: i32) -> Self::Return {
        if num % 2 == 0 {
            Eightball::Yes(Even(num))
        } else {
            Eightball::No
        }
    }
}

impl TryFrom2<i32> for Even2 {
    type Error = Eightball<!>;

    fn try_from2(num: i32) -> Result<Even2, Eightball<!>> {
        if num % 2 == 0 {
            Result::Ok(Even2(num))
        } else {
            Result::Err(Eightball::No)
        }
    }
}

fn even_string_own_try_type(num: i32) -> Eightball<String> {
    let n = Even::try_from2(num)?;
    let s = format!("{}", n.0);
    Eightball::Yes(s)
}

fn even_string_via_result(num: i32) -> Eightball<String> {
    let n = Even2::try_from2(num)?;
    let s = format!("{}", n.0);
    Eightball::Yes(s)
}

struct TryFromIntError; // Cannot instantiate a std::num::TryFromIntError

/// Non-breaking: this is identical (text) to std impl
impl TryFrom2<i8> for u8 {
    type Error = TryFromIntError;

    fn try_from2(u: i8) -> Result<Self, Self::Error> {
        if u >= 0 {
            Ok(u as Self)
        } else {
            Err(TryFromIntError)
        }
    }
}

fn main() {
    assert!(matches!(even_string_own_try_type(2), Eightball::Yes(s) if s == "2"));
    assert!(matches!(even_string_own_try_type(1), Eightball::No));

    assert!(matches!(even_string_via_result(2), Eightball::Yes(s) if s == "2"));
    assert!(matches!(even_string_via_result(1), Eightball::No));

    assert!(matches!(u8::try_from2(5), Ok(5)));
    assert!(matches!(u8::try_from2(-1), Err(TryFromIntError)));
}
