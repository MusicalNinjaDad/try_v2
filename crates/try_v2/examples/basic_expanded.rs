#![allow(unused)]
#![cfg_attr(unstable_never_type, feature(never_type))]
#![cfg_attr(unstable_try_trait_v2, feature(try_trait_v2))]
#![cfg_attr(unstable_try_trait_v2_residual, feature(try_trait_v2_residual))]
use std::process::Termination;

#[derive(Debug)]
#[must_use]
enum Exit<T: Termination> {
    Ok(T),
    Error(String),
    InvocationError(String),
}

// Recursive expansion of Try macro
// =================================

impl<T: Termination> std::ops::Try for Exit<T> {
    type Output = T;
    type Residual = Exit<!>;

    #[inline]
    fn from_output(output: Self::Output) -> Self {
        Self::Ok(output)
    }

    #[inline]
    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::Ok(v0) => std::ops::ControlFlow::Continue(v0),
            Self::Error(v0) => std::ops::ControlFlow::Break(Exit::Error(v0)),
            Self::InvocationError(v0) => std::ops::ControlFlow::Break(Exit::InvocationError(v0)),
        }
    }
}

impl<T: Termination> std::ops::FromResidual<Exit<!>> for Exit<T> {
    #[inline]
    #[track_caller]
    fn from_residual(residual: Exit<!>) -> Self {
        match residual {
            Exit::Error(v0) => Exit::Error(v0),
            Exit::InvocationError(v0) => Exit::InvocationError(v0),
        }
    }
}

impl<T: Termination> std::ops::Residual<T> for Exit<!> {
    type TryType = Exit<T>;
}

// =================================

impl<T: Termination> Termination for Exit<T> {
    fn report(self) -> std::process::ExitCode {
        todo!()
    }
}

fn main() {}
