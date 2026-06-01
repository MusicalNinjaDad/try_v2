use std::ops::{ControlFlow, FromResidual, Try};

pub trait Transform
where
    Self: Try + Sized,
{
    /// removes one level of nesting, converting `Foo<Foo<T>>` to `Foo<T>`
    /// or from `Foo<Bar<T>>` to `Bar<T>` if suitable residual interconversion is implemented.
    fn flatten<U>(self) -> U
    where
        Self: Try<Output = U>,
        U: FromResidual<Self::Residual>,
    {
        self?
    }

    /// converts from a `Foo<Bar<T>>` to a `Bar<Foo<T>>` where both `Foo` & `Bar` are `Try`.
    fn transpose<U, T>(self) -> U
    where
        Self::Output: Try<Output = T>,
        U: Try + FromResidual<<Self::Output as Try>::Residual>,
        U::Output: Try<Output = T> + FromResidual<Self::Residual>,
    {
        match self.branch() {
            ControlFlow::Continue(inner_u) => match inner_u.branch() {
                ControlFlow::Continue(val) => {
                    let inner_t = Try::from_output(val);
                    Try::from_output(inner_t)
                }
                ControlFlow::Break(u_residual) => FromResidual::from_residual(u_residual),
            },
            ControlFlow::Break(t_residual) => {
                let inner_t = FromResidual::from_residual(t_residual);
                Try::from_output(inner_t)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl<T> Transform for Option<T> {}
    impl<T, E> Transform for Result<T, E> {}

    mod flatten {
        use super::*;

        #[test]
        fn some_some() {
            let some_5 = Some(Some(5));
            let stdlib = some_5.flatten();
            let custom = Transform::flatten(some_5);
            assert_eq!(stdlib, custom)
        }
    }

    mod transpose {
        use super::*;
        #[test]
        fn ok_some() {
            let ok_some: Result<Option<u32>, String> = Ok(Some(5));
            let stdlib: Option<Result<u32, String>> = ok_some.clone().transpose();
            let custom: Option<Result<u32, String>> = Transform::transpose(ok_some);
            assert_eq!(stdlib, custom)
        }

        #[test]
        fn ok_none() {
            let ok_none: Result<Option<u32>, String> = Ok(None);
            let stdlib = ok_none.clone().transpose();
            let custom = Transform::transpose(ok_none);
            assert_eq!(stdlib, custom)
        }

        #[test]
        fn err() {
            let err: Result<Option<i32>, String> = Err("Oops".to_string());
            let stdlib = err.clone().transpose();
            let custom = Transform::transpose(err);
            assert_eq!(stdlib, custom)
        }

        #[test]
        fn some_ok() {
            let some_ok: Option<Result<u32, String>> = Some(Ok(5));
            let stdlib: Result<Option<u32>, String> = some_ok.clone().transpose();
            let custom: Result<Option<u32>, String> = Transform::transpose(some_ok);
            assert_eq!(stdlib, custom)
        }

        #[test]
        fn some_err() {
            let some_err: Option<Result<u32, String>> = Some(Err("Oops".to_string()));
            let stdlib = some_err.clone().transpose();
            let custom = Transform::transpose(some_err);
            assert_eq!(stdlib, custom)
        }

        #[test]
        fn none() {
            let none: Option<Result<u32, String>> = None;
            let stdlib = none.clone().transpose();
            let custom = Transform::transpose(none);
            assert_eq!(stdlib, custom)
        }
    }
}
