#![feature(never_type)]
#![feature(try_trait_v2)]

use try_v2::Try;

#[derive(Debug, Try)]
enum Exit<T, E> {
    Ok(T),
    TestsFailed,
    OtherError(E),
}

fn main() {}
