#![feature(never_type)]
#![feature(try_trait_v2)]

use try_v2::{Try, Try_ConvertResult};

#[derive(Debug, Try, Try_ConvertResult)]
enum MyResult<'t, 'e, T, E> {
    Ok(&'t T),
    Err(&'e E),
}

fn main() {}
