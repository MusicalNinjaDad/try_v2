#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2_derive::Try;

#[derive(Debug, Try)]
#[FromResidual(Result<_, Self::Residual>)]
#[allow(unused)]
#[must_use]
enum MultipleFields<T> {
    Ok(T),
    OtherError(String, String, i32),
}

fn main() {}
