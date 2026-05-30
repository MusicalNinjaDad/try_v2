#![feature(try_trait_v2)]
#![allow(clippy::disallowed_names)]

use std::ops::{FromResidual, Try};

#[allow(dead_code)]
trait Transpose<U>
where
    Self: Try,
{
    fn transpose2(self) -> U;
}

impl<T: Try<Output = TO, Residual = TR>, TO, TR, U: Try<Output = UO, Residual = UR>, UO, UR, O>
    Transpose<U> for T
where
    TO: Try<Output = O, Residual = UR>,
    UO: Try<Output = O, Residual = TR>,
{
    fn transpose2(self) -> U {
        // type T = Result<Option<u32>, String>;
        // type TR = <T as Try>::Residual; // Result<!, String>
        // type U = Option<Result<u32, String>>;
        // type UO = <U as Try>::Output; // Result<u32, String>
        // type UR = <U as Try>::Residual; // Option<!>
        let opt_or_err = self.branch();
        match opt_or_err {
            std::ops::ControlFlow::Continue(opt) => match opt.branch() {
                std::ops::ControlFlow::Continue(val) => {
                    let inner_result = UO::from_output(val);
                    U::from_output(inner_result)
                }
                std::ops::ControlFlow::Break(opt_residual) => {
                    <U as FromResidual<UR>>::from_residual(opt_residual)
                }
            },
            std::ops::ControlFlow::Break(err) => {
                let inner_result = <UO as FromResidual<TR>>::from_residual(err);
                U::from_output(inner_result)
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
fn some_ok() {
    let some_ok: Option<Result<u32, String>> =Some(Ok(5));
    let stdlib: Result<Option<u32>, String> = some_ok.clone().transpose();
    let custom: Result<Option<u32>, String> = some_ok.transpose2();
    assert_eq!(stdlib, custom)
}

fn main() {}
