#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use std::{io, process::Termination};

use try_v2_derive::{FromResidual, Try};

#[derive(Debug, Try, FromResidual)]
#[FromResidual(Result)]
#[must_use]
enum Exit<T, E> {
    Ok(T),
    Error(E),
    InvocationError,
    IOError(io::Error),
}

impl<E> From<io::Error> for Exit<!, E> {
    fn from(err: io::Error) -> Self {
        Self::IOError(err)
    }
}

fn main() -> Exit<(), String> {
    Err(io::Error::other("error"))?;
    Exit::Ok(())
}

impl Termination for Exit<(), String> {
    fn report(self) -> std::process::ExitCode {
        todo!()
    }
}
