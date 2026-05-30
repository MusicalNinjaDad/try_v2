#![feature(try_trait_v2)]
#![allow(dead_code)]
#![allow(clippy::disallowed_names)]

use std::ops::{ControlFlow, FromResidual, Try};

trait Foo: Iterator {
    fn try_reduce2<F, R, U>(
        &mut self,
        f: F,
    ) -> U
    where
        Self: Sized,
        U: Try<Output = Option<Self::Item>> + FromResidual<R::Residual>,
        F: FnMut(Self::Item, Self::Item) -> R,
        R: Try<Output = Self::Item>,
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
