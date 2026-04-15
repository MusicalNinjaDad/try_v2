#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2::{Try, Try_ConvertResult};

#[derive(Debug, Try, Try_ConvertResult)]
#[must_use]
enum Exit<T, E> {
    Ok(T),
    TestsFailed,
    OtherError(T, E),
}

fn main() {}
