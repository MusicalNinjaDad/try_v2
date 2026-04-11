#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2::Try;

#[derive(Try)]
#[must_use]
enum ContainsTwoGenerics<T, E> {
    R(Result<T, E>),
    O(Option<T>),
}

#[derive(Try)]
#[must_use]
enum ContainsWrongGenerics<T, U, E> {
    R(Result<U, E>),
    O(Option<T>),
}

#[derive(Try)]
#[must_use]
enum OutputAndBangIsOK<T, U> {
    R(Result<U, !>),
    O(Option<T>),
}

fn main() {}
