#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use std::process::Termination;

use try_v2_derive::Try;

#[derive(Debug, Try)]
#[FromResidual(Result<_, Self::Residual>)]
#[allow(unused)] // If it compiles then it already passes
#[must_use]
enum NoUnitResiduals<T: Termination> {
    Ok(T),
    OtherError(String),
}

fn main() {}
