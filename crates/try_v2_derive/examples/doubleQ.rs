#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]
#![feature(min_specialization)]
#![allow(unused, clippy::disallowed_names)]
use std::{io, ops::ControlFlow};

fn wibble() -> io::Result<Option<()>> {
    // let _ = Ok(Some(3))??;
    Ok(Some(()))
}

#[derive(Debug)]
#[must_use]
enum Foo<T> {
    Out(T),
    Res,
}

pub trait Try: FromResidual {
    type Output;
    type Residual;

    fn from_output(output: Self::Output) -> Self;
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output>;
}

pub trait FromResidual<R = <Self as Try>::Residual> {
    // Required method
    fn from_residual(residual: R) -> Self;
}

pub trait Residual<O>: Sized {
    type TryType: Try<Output = O, Residual = Self>;
}

impl<T> Try for Foo<T> {
    type Output = T;
    type Residual = Foo<!>;
    #[inline]
    fn from_output(output: Self::Output) -> Self {
        todo!()
    }
    #[inline]
    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        todo!()
    }
}

impl<T> FromResidual<Foo<!>> for Foo<T> {
    #[inline]
    #[track_caller]
    default fn from_residual(residual: Foo<!>) -> Self {
        todo!()
    }
}

impl<T> Residual<T> for Foo<!> {
    type TryType = Foo<T>;
}

// // Currently not possible as min_specialization does not consider type parameters, see last e.g.:
// // https://github.com/rust-lang/rfcs/blob/master/text/1210-impl-specialization.md#extending-hrtbs
// impl<T,X,Y> FromResidual<Y::Residual> for X
// where
// X: Try<Output = Y>, // `Foo<Bar<T>>`
// Y: Try<Output = T>, // `Bar<T>`
// {
//     fn from_residual(residual: Y::Residual) -> Self {
//         Try::from_output(FromResidual::from_residual(residual))
//     }
// }

struct OrdOnly<T: Ord>(T);

trait SpecTrait<U> {
    fn f();
}

impl<T, U> SpecTrait<U> for T {
    default fn f() {}
}

impl<T: Ord> SpecTrait<()> for OrdOnly<T> {
    fn f() {}
}

impl<T: Ord> SpecTrait<OrdOnly<T>> for () {
    fn f() {}
}

impl<T: Ord, U: Ord, V: Ord> SpecTrait<(OrdOnly<T>, OrdOnly<U>)> for &[OrdOnly<V>] {
    fn f() {}
}

fn main() {}
