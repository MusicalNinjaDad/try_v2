#![feature(never_type)]
#![feature(try_trait_v2)]

use std::process::Termination;

use try_v2::{Try, Try_ConvertResult};

#[derive(Debug, Try, Try_ConvertResult)]
#[allow(unused)] // If it compiles then it already passes
enum NoUnitResiduals<T: Termination> {
    Ok(T),
    OtherError(String),
}

fn main() {}
