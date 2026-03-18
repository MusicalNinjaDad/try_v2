#![feature(never_type)]
#![feature(try_trait_v2)]

use try_v2::Try;

#[derive(Try)]
enum NoGenerics {
    Ok,
    TestsFailed,
    OtherError(String),
}

#[derive(Try)]
enum GenericNotOutput<T, E> {
    Ok(E),
    Err(T),
}

#[derive(Try)]
enum TooManyOutputs<T, E> {
    Ok(T, E),
    Err,
}

fn main() {}
