#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]
#![feature(iterator_try_reduce)]
#![allow(dead_code)]
#![allow(clippy::disallowed_names)]

use std::ops::{ControlFlow, FromResidual, Residual, Try};

trait Foo: Iterator {
    fn try_reduce2<R, U>(&mut self, f: impl FnMut(Self::Item, Self::Item) -> R) -> U
    where
        Self: Sized,
        R: Try<Output = Self::Item>,
        // U is the *canonical* TryType from R::Residual, without this foo.try_reduce(...)? is ambiguous
        R::Residual: Residual<Option<Self::Item>, TryType = U>,
        U: Try<Output = Option<Self::Item>> + FromResidual<R::Residual>,
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

impl<I: Iterator> Foo for I {}

fn immediate_qmark() -> Result<Option<i32>, i32> {
    let nums: [i32; 3] = [1, 2, 3];
    let sum_positive = |acc, n| match n {
        ..0 => Err(n),
        _ => Ok(acc + n),
    };
    let _sum = nums.into_iter().try_reduce(sum_positive)?;
    let sum2 = nums.into_iter().try_reduce2(sum_positive)?;
    Ok(sum2)
}

fn main() {}
