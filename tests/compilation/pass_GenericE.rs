#![feature(never_type)]
#![feature(try_trait_v2)]

use try_v2::Try;

#[derive(Debug, Try)]
enum ExitE<E> {
    Ok(E),
    TestsFailed,
    OtherError(String),
}

fn main() {}
