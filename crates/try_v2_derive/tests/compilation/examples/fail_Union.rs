#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2_derive::{Try, Try_ConvertResult};

#[derive(Try, Try_ConvertResult)]
#[must_use]
union Union {
    foo: u8,
    bar: u8,
}

fn main() {}
