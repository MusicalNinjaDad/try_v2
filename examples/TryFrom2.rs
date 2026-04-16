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

trait TryFrom2<T> {
    type Return: std::ops::Try;

    fn try_from2(value: T) -> Self::Return;
}

impl TryFrom2<i32> for Even {
    type Return = Eightball<Self>;

    fn try_from2(num: i32) -> Self::Return {
        if num % 2 == 0 {
            Eightball::Yes(Even(num))
        } else {
            Eightball::No
        }
    }
}

fn even_string2(num: i32) -> Eightball<String> {
    let n = Even::try_from2(num)?;
    let s = format!("{}", n.0);
    Eightball::Yes(s)
}

trait TryFrom3<T>: std::marker::Sized {
    type Error;
    type Return: std::ops::Try = Result<Self, Self::Error>;

    fn try_from3(value: T) -> Self::Return;
}

impl TryFrom3<i32> for Even {
    type Error = ();
    type Return = Eightball<Self>;

    fn try_from3(num: i32) -> Self::Return {
        if num % 2 == 0 {
            Eightball::Yes(Even(num))
        } else {
            Eightball::No
        }
    }
}

impl TryFrom3<i32> for Even2 {
    type Error = Eightball<!>;

    fn try_from3(num: i32) -> Result<Even2, Eightball<!>> {
        if num % 2 == 0 {
            Result::Ok(Even2(num))
        } else {
            Result::Err(Eightball::No)
        }
    }
}

fn even_string3(num: i32) -> Eightball<String> {
    let n = Even::try_from3(num)?;
    let s = format!("{}", n.0);
    Eightball::Yes(s)
}

fn even_string3_via_result(num: i32) -> Eightball<String> {
    let n = Even2::try_from3(num)?;
    let s = format!("{}", n.0);
    Eightball::Yes(s)
}

fn main() {
    assert!(matches!(even_string(2), Eightball::Yes(s) if s == "2"));
    assert!(matches!(even_string(1), Eightball::No));

    assert!(matches!(even_string2(2), Eightball::Yes(s) if s == "2"));
    assert!(matches!(even_string2(1), Eightball::No));

    assert!(matches!(even_string3(2), Eightball::Yes(s) if s == "2"));
    assert!(matches!(even_string3(1), Eightball::No));

    assert!(matches!(even_string3_via_result(2), Eightball::Yes(s) if s == "2"));
    assert!(matches!(even_string3_via_result(1), Eightball::No));
}
