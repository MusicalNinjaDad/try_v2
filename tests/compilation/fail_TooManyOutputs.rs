#![feature(never_type)]
#![feature(try_trait_v2)]

use try_v2::{Try, Try_ConvertResult};

#[derive(Try, Try_ConvertResult)]
#[must_use]
enum TooManyOutputs<T, E> {
    Ok(T, E),
    Err,
}

#[derive(Try, Try_ConvertResult)]
#[must_use]
enum TooManyOutputsBorrowed<'t, 'e, T, E> {
    Ok(&'t T, &'e E),
    Err,
}

#[derive(Try, Try_ConvertResult)]
#[must_use]
enum TooManyOutputsBorrowedOrdering<'e, 't, T, E> {
    Ok(&'e E, &'t T),
    Err,
}

fn main() {}
