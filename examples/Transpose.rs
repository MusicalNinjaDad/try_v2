#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]
#![allow(clippy::disallowed_names)]

use std::ops::Try;

trait Transpose<U>
where
    Self: Try,
{
    fn transpose2(self) -> U;
}

impl Transpose<Option<Result<u32, String>>> for Result<Option<u32>, String> {
    fn transpose2(self) -> Option<Result<u32, String>> {
        let opt_or_err = self.branch();
        match opt_or_err {
            std::ops::ControlFlow::Continue(opt) => match opt {
                Some(val) => Some(Ok(val)),
                None => None,
            },
            std::ops::ControlFlow::Break(err) => {
                let Err(err) = err;
                Some(Err(err))
            },
        }
    }
}

fn main() {}
