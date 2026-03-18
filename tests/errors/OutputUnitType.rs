#![feature(never_type)]
#![feature(try_trait_v2)]

use try_v2::Try;

#[derive(Try)]
enum OutputUnitType<T> {
    Ok,
    TestsFailed(T),
    OtherError(String),
}

fn main() {}
