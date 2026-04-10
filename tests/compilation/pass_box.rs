#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2::Try;

#[derive(Try)]
#[allow(unused)]
#[must_use]
enum Eightball<Y> {
    Yes(Box<Y>),
    No,
}

fn main() {}
