#![feature(try_trait_v2)]
#![allow(unused)]
use std::ops::{FromResidual, Try};

// impl<T,X,Y> FromResidual<Y::Residual> for X
// where
// X: Try<Output = Y>, // `Foo<Bar<T>>`
// Y: Try<Output = T>, // `Bar<T>`
// {
//     fn from_residual(residual: Y::Residual) -> Self {
//         Try::from_output(FromResidual::from_residual(residual))
//     }
// }

fn main() {}
