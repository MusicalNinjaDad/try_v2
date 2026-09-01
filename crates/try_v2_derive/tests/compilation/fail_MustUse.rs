#![allow(stable_features)]
#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2_derive::Try;

#[derive(Debug, Try)]
enum ExitE<E> {
    Ok(E),
    TestsFailed,
    OtherError(String),
}

fn main() {
    fail
}
