#![feature(never_type)]
#![feature(try_trait_v2)]

use try_v2::{Try, Try_ConvertResult};

#[derive(Debug, Try, Try_ConvertResult)]
enum ExitE<E> {
    Ok(E),
    TestsFailed,
    OtherError(String),
}

fn main() {
    fail
}
