#![feature(never_type)]
#![feature(try_trait_v2)]

use try_v2::Try;

#[derive(Try)]
enum GenericNotOutput<T, E> {
    Ok(E),
    Err(T),
}

fn main() {}
