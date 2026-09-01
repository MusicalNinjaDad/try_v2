#![cfg_attr(unstable_never_type, feature(never_type))]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use std::process::Termination;

use try_v2_derive::Try;

#[derive(Debug, Try)]
#[FromResidual(Result<_, Self::Residual>)]
#[must_use]
#[allow(unused)] // If it compiles then it already passes
enum NoFieldResiduals<T: Termination> {
    Ok(T),
    TestsFailed,
}

fn main() {}
