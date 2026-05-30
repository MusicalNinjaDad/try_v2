#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]
#![allow(dead_code)]
#![allow(clippy::disallowed_names)]

use std::ops::{ControlFlow, FromResidual, Residual, Try};

trait Foo: Iterator {
    fn try_reduce2<F, R>(
        &mut self,
        f: F,
    ) -> <<R as Try>::Residual as Residual<Option<<R as Try>::Output>>>::TryType
    where
        Self: Sized,
        F: FnMut(Self::Item, Self::Item) -> R,
        R: Try<Output = Self::Item>,
        <R as Try>::Residual: Residual<Option<Self::Item>>,
    {
        let first = match self.next() {
            Some(i) => i,
            None => return Try::from_output(None),
        };

        match self.try_fold(first, f).branch() {
            ControlFlow::Break(r) => FromResidual::from_residual(r),
            ControlFlow::Continue(i) => Try::from_output(Some(i)),
        }
    }
}

fn main() {}
