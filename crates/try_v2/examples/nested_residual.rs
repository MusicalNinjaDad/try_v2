#![cfg_attr(unstable_never_type, feature(never_type))]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]
#![allow(dead_code, clippy::disallowed_names)]

use std::ops::FromResidual;

use try_v2_derive::Try;

#[derive(Debug, Try)]
#[must_use]
enum Known<T> {
    Known(T),
    Unknown,
}

#[derive(Debug, Try)]
#[must_use]
enum Good<T, B> {
    Good(T),
    Bad(B),
}

impl<B, T> FromResidual<Good<!, B>> for Known<Good<T, B>> {
    fn from_residual(_residual: Good<!, B>) -> Self {
        todo!()
    }
}

fn wibble() -> Known<Good<!, i32>> {
    todo!()
}

// ?? solves the Poll<Result<!>> problem
fn wobble() -> Known<Good<i32, i32>> {
    wibble()??
}

fn main() {}
