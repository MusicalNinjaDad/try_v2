#![feature(never_type)]
#![feature(try_trait_v2)]

use try_v2::Try;

#[derive(Debug, Try)]
#[allow(unused)]
enum MultipleFields<T> {
    Ok(T),
    OtherError(String, String, i32),
}

fn main() {}
