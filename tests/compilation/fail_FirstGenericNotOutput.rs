#![feature(never_type)]
#![feature(try_trait_v2)]

use try_v2::Try;

#[derive(Try)]
enum FirstGenericNotOutput<T, E> {
    Ok(E),
    Err(T),
}

#[derive(Try)]
enum FirstGenericNotOutputBorrowed<'t, 'e, T, E> {
    Ok(&'e E),
    Err(&'t T),
}

fn main() {}
