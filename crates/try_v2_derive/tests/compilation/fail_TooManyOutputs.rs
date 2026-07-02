#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2_derive::Try;

#[derive(Debug, Try)]
#[FromResidual(Result<_, Self::Residual>)]
enum TooManyOutputs<T, E> {
    Ok(T, E),
    Err,
}

#[derive(Debug, Try)]
#[FromResidual(Result<_, Self::Residual>)]
#[must_use]
enum TooManyOutputsBorrowed<'t, 'e, T, E> {
    Ok(&'t T, &'e E),
    Err,
}

#[derive(Debug, Try)]
#[FromResidual(Result<_, Self::Residual>)]
#[must_use]
enum TooManyOutputsBorrowedOrdering<'e, 't, T, E> {
    Ok(&'e E, &'t T),
    Err,
}

fn main() {}
