#![feature(never_type)]
#![feature(try_trait_v2)]

use try_v2::{Try, Try_ConvertResult};

#[derive(Try, Try_ConvertResult)]
enum Owned<T, E> {
    Ok(E),
    Err(T),
}

#[derive(Try, Try_ConvertResult)]
enum Borrowed<'t, 'e, T, E> {
    Ok(&'e E),
    Err(&'t T),
}

#[derive(Try, Try_ConvertResult)]
enum MultipleBorrowed<'t, 'e, 'f, T, E, F> {
    Ok(&'e E, &'f F),
    Err(&'t T),
}

fn main() {}
