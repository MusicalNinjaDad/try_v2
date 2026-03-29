#![feature(never_type)]
#![feature(try_trait_v2)]

use try_v2::Try;

#[derive(Try)]
enum Owned<T, E> {
    Ok(E),
    Err(T),
}

#[derive(Try)]
enum Borrowed<'t, 'e, T, E> {
    Ok(&'e E),
    Err(&'t T),
}

#[derive(Try)]
enum MultipleBorrowed<'t, 'e, 'f, T, E, F> {
    Ok(&'e E, &'f F),
    Err(&'t T),
}

fn main() {}
