#![allow(stable_features)]
#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2_derive::Try;

#[derive(Debug, Try)]
#[FromResidual(Result<_, Self::Residual>)]
enum Owned<T, E> {
    Ok(E),
    Err(T),
}

#[derive(Debug, Try)]
#[FromResidual(Result<_, Self::Residual>)]
#[must_use]
enum Borrowed<'t, 'e, T, E> {
    Ok(&'e E),
    Err(&'t T),
}

#[derive(Debug, Try)]
#[FromResidual(Result<_, Self::Residual>)]
#[must_use]
enum MultipleBorrowed<'t, 'e, 'f, T, E, F> {
    Ok(&'e E, &'f F),
    Err(&'t T),
}

fn main() {}
