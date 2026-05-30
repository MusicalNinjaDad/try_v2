#![feature(try_trait_v2)]

use std::ops::{ControlFlow, FromResidual, Try};

#[allow(dead_code)]
/// Converts from a `Foo<Bar<T>>` to a `Bar<Foo<T>>` where both `Foo` & `Bar` are `Try`.
///
/// The blanket impl provides e.g. `Result<Option<T>,E>` <-> `Option<Result<T,E>>`
/// which are currently hand-rolled in the stdlib along with equivalent functionality for
/// *all* `Try` types.
///
/// The blanket impl means that no custom impl is possible. This trait, if in scope, applies
/// automatically to all possible cases.
/// (But is borken for `Poll` due to the wonky `Try` implementation)
trait Transpose<U, O>
where
    Self: Try,
    Self::Output: Try<Output = O, Residual = U::Residual>,
    U: Try,
    U::Output: Try<Output = O, Residual = Self::Residual>,
{
    fn transpose2(self) -> U;
}

impl<T, U, O> Transpose<U, O> for T
where
    Self: Try,
    Self::Output: Try<Output = O, Residual = U::Residual>,
    U: Try,
    U::Output: Try<Output = O, Residual = Self::Residual>,
{
    fn transpose2(self) -> U {
        match self.branch() {
            ControlFlow::Continue(inner_u) => match inner_u.branch() {
                ControlFlow::Continue(val) => {
                    let inner_t = Try::from_output(val);
                    U::from_output(inner_t)
                }
                ControlFlow::Break(u_residual) => U::from_residual(u_residual),
            },
            ControlFlow::Break(t_residual) => {
                let inner_t = FromResidual::from_residual(t_residual);
                U::from_output(inner_t)
            }
        }
    }
}

#[test]
fn ok_some() {
    let ok_some: Result<Option<u32>, String> = Ok(Some(5));
    let stdlib: Option<Result<u32, String>> = ok_some.clone().transpose();
    let custom: Option<Result<u32, String>> = ok_some.transpose2();
    assert_eq!(stdlib, custom)
}

#[test]
fn ok_none() {
    let ok_none: Result<Option<u32>, String> = Ok(None);
    let stdlib = ok_none.clone().transpose();
    let custom = ok_none.transpose2();
    assert_eq!(stdlib, custom)
}

#[test]
fn err() {
    let err: Result<Option<i32>, String> = Err("Oops".to_string());
    let stdlib = err.clone().transpose();
    let custom = err.transpose2();
    assert_eq!(stdlib, custom)
}

#[test]
fn some_ok() {
    let some_ok: Option<Result<u32, String>> = Some(Ok(5));
    let stdlib: Result<Option<u32>, String> = some_ok.clone().transpose();
    let custom: Result<Option<u32>, String> = some_ok.transpose2();
    assert_eq!(stdlib, custom)
}

#[test]
fn some_err() {
    let some_err: Option<Result<u32, String>> = Some(Err("Oops".to_string()));
    let stdlib = some_err.clone().transpose();
    let custom = some_err.transpose2();
    assert_eq!(stdlib, custom)
}

#[test]
fn none() {
    let none: Option<Result<u32, String>> = None;
    let stdlib = none.clone().transpose();
    let custom = none.transpose2();
    assert_eq!(stdlib, custom)
}

fn main() {}
