#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2::{Try,Try_ConvertResult};

#[derive(Try, Try_ConvertResult)]
#[must_use]
enum Eightball<T> {
    Yes(T),
    No,
}

fn even(num: i32) -> Result<i32, Eightball<!>> {
    if num % 2 == 0 {
        Result::Ok(num)
    } else {
        Result::Err(Eightball::No)
    }
}

fn even_string(num: i32) -> Eightball<String> {
    let n = even(num)?;
    let s = format!("{n}");
    Eightball::Yes(s)
}

fn main() {}
