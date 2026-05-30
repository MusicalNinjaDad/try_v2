#![feature(try_trait_v2)]
#![allow(dead_code)]

use std::fmt::Debug;
use std::ops::{ControlFlow, Try};

trait Unwrap
where
    Self: Try,
{
    fn unwrap2(self) -> Self::Output;
    fn expect2(self) -> Self::Output;
}

impl<T: Try> Unwrap for T
where
    T::Residual: Debug,
{
    fn unwrap2(self) -> Self::Output {
        match self.branch() {
            ControlFlow::Continue(output) => output,
            ControlFlow::Break(residual) => panic!("called unwrap on a residual value: {residual:?}"),
        }
    }

    fn expect2(self) -> Self::Output {
        todo!()
    }
}

fn main() {}
