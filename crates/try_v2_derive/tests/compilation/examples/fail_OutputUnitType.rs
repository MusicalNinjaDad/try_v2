#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2_derive::{Try, Try_ConvertResult};

#[derive(Try, Try_ConvertResult)]
#[must_use]
enum OutputUnitType<T> {
    Ok,
    TestsFailed(T),
    OtherError(String),
}

fn main() {}
