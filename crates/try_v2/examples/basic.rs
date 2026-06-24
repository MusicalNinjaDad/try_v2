#![allow(unused)]
#![cfg_attr(unstable_never_type, feature(never_type))]
#![cfg_attr(unstable_try_trait_v2, feature(try_trait_v2))]
#![cfg_attr(unstable_try_trait_v2_residual, feature(try_trait_v2_residual))]
use std::process::Termination;
use try_v2::Try;

#[derive(Debug, Try)]
#[must_use]
enum Exit<T: Termination> {
    Ok(T),
    Error(String),
    InvocationError(String),
}

impl<T: Termination> Termination for Exit<T> {
    fn report(self) -> std::process::ExitCode {
        todo!()
    }
}

fn main() {}
