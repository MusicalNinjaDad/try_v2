#![feature(never_type)]
#![feature(try_trait_v2)]

use try_v2::{Try, Try_ConvertResult};

#[derive(Try, Try_ConvertResult)]
enum OutputNamedField<T> {
    Ok{foo: T},
    TestsFailed(T),
    OtherError(String),
}

#[derive(Try, Try_ConvertResult)]
enum OutputNamedFieldBorrowed<'t, T> {
    Ok{foo: &'t T},
    TestsFailed(T),
    OtherError(String),
}

fn main() {}
