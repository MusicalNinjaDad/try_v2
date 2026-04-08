#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2::{Try, Try_ConvertResult};

#[derive(Try, Try_ConvertResult)]
#[must_use]
enum Owned<T, E> {
    Ok(E),
    Err(T),
}

#[derive(Try, Try_ConvertResult)]
#[must_use]
enum Borrowed<'t, 'e, T, E> {
    Ok(&'e E),
    Err(&'t T),
}

#[derive(Try, Try_ConvertResult)]
#[must_use]
enum MultipleBorrowed<'t, 'e, 'f, T, E, F> {
    Ok(&'e E, &'f F),
    Err(&'t T),
}

fn main() {}
