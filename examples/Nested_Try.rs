#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]
#![allow(dead_code, clippy::disallowed_names)]

use std::{error::Error, io};

use try_v2::Try;

use DuplicateData::Duplicate;

#[derive(Debug, Try)]
#[must_use]
enum DuplicateData<T> {
    Duplicate(T),
    NoCandidate,
    NoDuplicates,
    ParsingError(Box<dyn Error>),
    IOError(Box<io::Error>),
}

fn main() {
    fn process(foo: DuplicateData<i32>) -> DuplicateData<i32> {
        let baz = foo? + 1;
        Duplicate(baz)
    }
    let foo: DuplicateData<i32> = Duplicate(5);
    assert!(matches!(process(foo), Duplicate(6)));
}
