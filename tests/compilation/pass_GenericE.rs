#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2::{Try, Try_ConvertResult};

#[derive(Debug, Try, Try_ConvertResult)]
#[must_use]
enum ExitE<E> {
    Ok(E),
    TestsFailed,
    OtherError(String),
}

fn main() {}
