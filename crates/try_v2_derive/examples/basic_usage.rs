#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use std::{io, process::Termination};

use try_v2_derive::Try;

#[derive(Debug, Try)]
#[must_use]
enum Exit<T, E> {
    Ok(T),
    Error(E),
    InvocationError,
    IOError(io::Error),
}

fn main() -> Exit<i32, String> {
    todo!()
}

impl Termination for Exit<i32, String> {
    fn report(self) -> std::process::ExitCode {
        todo!()
    }
}
