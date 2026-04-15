#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]
#![allow(dead_code)]

use std::fmt::Debug;

use try_v2::{Try, Try_Methods};

#[derive(Debug, Try, Try_Methods)]
#[must_use]
enum ExitE<T: std::fmt::Debug, E: Debug, F> {
    Ok(T),
    TestsFailed(F),
    OtherError(E),
}

fn main() {}
