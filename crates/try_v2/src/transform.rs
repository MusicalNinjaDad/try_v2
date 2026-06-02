use std::ops::{ControlFlow, FromResidual, Residual, Try};

pub trait Transform
where
    Self: Try + Sized,
{
    /// Removes one level of nesting, converting `Foo<Foo<T>>` to `Foo<T>`
    /// or from `Foo<Bar<T>>` to `Bar<T>` if suitable residual interconversion is implemented.
    fn flatten<U>(self) -> U
    where
        Self: Try<Output = U>,
        U: FromResidual<Self::Residual>,
    {
        self?
    }

    /// Calls a function with a reference to the contained value. Returns the original Self
    fn inspect<F>(self, f: F) -> Self
    where
        F: FnOnce(&Self::Output),
    {
        let val = self?;
        f(&val);
        Try::from_output(val)
    }

    /// Applys a function to the contained value converting `T` -> `U` then
    /// returns the canonical TryType for Self with Output `U`
    fn map<X, U, F>(self, f: F) -> X
    where
        F: FnOnce(Self::Output) -> U,
        X: Try<Output = U> + FromResidual<Self::Residual>,
        Self::Residual: Residual<U, TryType = X>,
    {
        let val = self?;
        let mapped = f(val);
        Try::from_output(mapped)
    }

    fn map_or<U, F>(self, default: U, f: F) -> U
    where
        F: FnOnce(Self::Output) -> U,
    {
        match self.branch() {
            ControlFlow::Continue(val) => f(val),
            ControlFlow::Break(_) => default,
        }
    }

    fn map_or_else<U, D, F>(self, default: D, f: F) -> U
    where
        D: FnOnce() -> U,
        F: FnOnce(Self::Output) -> U,
    {
        match self.branch() {
            ControlFlow::Continue(val) => f(val),
            ControlFlow::Break(_) => default(),
        }
    }

    /// Converts from a `Foo<Bar<T>>` to a `Bar<Foo<T>>` where both `Foo` & `Bar` are `Try`.
    fn transpose<U, T, BART, FOOT>(self) -> U
    where
        Self: Try<Output = BART>,
        BART: Try<Output = T>,
        FOOT: Try<Output = T> + FromResidual<Self::Residual>,
        U: Try<Output = FOOT> + FromResidual<BART::Residual>,
        BART::Residual: Residual<FOOT, TryType = U>,
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

    fn zip<U, Z>(self, other: U) -> Z
    where
        U: Try,
        Z: Try<Output = (Self::Output, U::Output)>
            + FromResidual<Self::Residual>
            + FromResidual<U::Residual>,
        Self::Residual: Residual<Z::Output, TryType = Z>,
    {
        let v1 = self?;
        let v2 = other?;
        Try::from_output((v1, v2))
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

    mod inspect {
        use super::*;
        use std::fmt::Write;

        #[test]
        fn some_5() {
            let some_5 = Some(5);
            let mut text = String::new();
            some_5.inspect(|x| write!(text, "{x}").expect("failed to write {x} to text"));
            assert_eq!(text, "5");
            Transform::inspect(some_5, |x| {
                write!(text, "{x}").expect("failed to write {x} to text")
            });
            assert_eq!(text, "55");
        }
    }

    mod map {
        use super::*;

        #[test]
        fn map() {
            let some_5 = Some(5);
            let stdlib = some_5.map(|x| x + 1);
            let custom = Transform::map(some_5, |x| x + 1);
            assert_eq!(stdlib, custom);
        }

        #[test]
        fn map_or_some() {
            let some_5 = Some(5);
            let stdlib = some_5.map_or(0, |x| x + 1);
            let custom = Transform::map_or(some_5, 0, |x| x + 1);
            assert_eq!(stdlib, custom);
        }

        #[test]
        fn map_or_none() {
            let some_5: Option<u32> = None;
            let stdlib = some_5.map_or(0, |x| x + 1);
            let custom = Transform::map_or(some_5, 0, |x| x + 1);
            assert_eq!(stdlib, custom);
        }

        #[test]
        fn map_or_else_some() {
            let some_5 = Some(5);
            let stdlib = some_5.map_or_else(|| 1 + 1, |x| x + 1);
            let custom = Transform::map_or_else(some_5, || 1 + 1, |x| x + 1);
            assert_eq!(stdlib, custom);
        }

        #[test]
        fn map_or_else_none() {
            let some_5: Option<u32> = None;
            let stdlib = some_5.map_or_else(|| 1 + 1, |x| x + 1);
            let custom = Transform::map_or_else(some_5, || 1 + 1, |x| x + 1);
            assert_eq!(stdlib, custom);
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
            // assert_eq!(stdlib, custom)
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

    mod zip {
        use super::*;

        #[test]
        fn some_some() {
            let some_1 = Some(1);
            let some_x = Some("x");
            let stdlib = some_1.zip(some_x);
            let custom = Transform::zip(some_1, some_x);
            assert_eq!(stdlib, custom);
        }
    }
}
