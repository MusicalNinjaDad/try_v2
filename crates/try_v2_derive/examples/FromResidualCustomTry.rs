#![allow(unused)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use std::{num::NonZeroI32, ops::{FromResidual, Residual, Try}};

use try_v2_derive::FromResidual;

#[derive(Debug, FromResidual, PartialEq)]
#[FromResidual(Option<Self>)]
#[must_use]
struct ResultCode(i32);
impl ResultCode {
    const SUCCESS: Self = ResultCode(0);
}

struct ErrorCode(NonZeroI32);

impl Try for ResultCode {
    type Output = i32;

    type Residual = ErrorCode;

    fn from_output(output: Self::Output) -> Self {
        todo!()
    }

    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        todo!()
    }
}

impl FromResidual<ErrorCode> for ResultCode {
    fn from_residual(residual: ErrorCode) -> Self {
        todo!()
    }
}

impl Residual<i32> for ErrorCode {
    type TryType = ResultCode;
}

fn maybe_code() -> Option<ResultCode> {
    let _ = ResultCode(3)?;
    None
}

fn main() {}
