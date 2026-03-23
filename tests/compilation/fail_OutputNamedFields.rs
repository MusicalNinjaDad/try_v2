#![feature(never_type)]
#![feature(try_trait_v2)]

use try_v2::Try;

#[derive(Try)]
enum OutputNamedField<T> {
    Ok{foo: T},
    TestsFailed(T),
    OtherError(String),
}

fn main() {}
