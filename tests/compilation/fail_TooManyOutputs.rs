#![feature(never_type)]
#![feature(try_trait_v2)]

use try_v2::Try;

#[derive(Try)]
enum TooManyOutputs<T, E> {
    Ok(T, E),
    Err,
}

#[derive(Try)]
enum TooManyOutputsBorrowed<'t, 'e, T, E> {
    Ok(&'t T, &'e E),
    Err,
}

fn main() {}
