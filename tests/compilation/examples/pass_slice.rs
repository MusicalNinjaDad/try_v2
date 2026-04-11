#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2::{Try, Try_ConvertResult};

#[derive(Try, Try_ConvertResult)]
#[must_use]
enum Eightball<'y, Y> {
    Yes(&'y [Y]),
    No,
}

fn main() {}
