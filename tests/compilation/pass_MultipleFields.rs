#![feature(never_type)]
#![feature(try_trait_v2)]

use try_v2::{Try, Try_ConvertResult};

#[derive(Debug, Try, Try_ConvertResult)]
#[allow(unused)]
#[must_use]
enum MultipleFields<T> {
    Ok(T),
    OtherError(String, String, i32),
}

fn main() {}
