#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2_derive::Try;

#[derive(Debug, Try)]
#[FromResidual(Result<_, Self::Residual>)]
enum OutputNamedField<T> {
    Ok { foo: T },
    TestsFailed(T),
    OtherError(String),
}

#[derive(Debug, Try)]
#[FromResidual(Result<_, Self::Residual>)]
#[must_use]
enum OutputNamedFieldBorrowed<'t, T> {
    Ok { foo: &'t T },
    TestsFailed(T),
    OtherError(String),
}

fn main() {}
