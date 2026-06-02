use std::ops::{ControlFlow, FromResidual, Residual, Try};

/// Methods for transforming TryTypes. Inspired by the methods provided on `Option` & `Result`
///
/// ## Note
///
/// - Methods which act on the contained value are only available for the *Output* case. TryTypes
///   are recommended to directly implement equivalent methods for *Residual* cases with suitable
///   naming. E.g. we provide a `.map()` but not a `.map_err()` equivalent as multiple such
///   methods may be needed and no standardised naming makes sense.
/// - Methods which act on the contained value will extract a value of type `Output` and return the
///   *canonical TryType* for the new Output. This is identifiable by the generic type `X` in the
///   method signature. This is usually the expected behaviour but can lead to a different value
///   type or resulting TryType where `Try` is not implemented symmetrically.
/// - Generic type conventions used in signatures (in standard order):
///     - `X` the *canonical TryType* returned
///     - `V` the other TryType
///     - `T` the `Output` type for `Self`
///     - `U` the other `Output` type
///     - `F` a function/closure passed as a parameter
///     - `G` the return type of `F`
///     - `R` *never used* to avoid confusion with "Residual".
pub trait Transform
where
    Self: Try + Sized,
{
    /// Removes one level of nesting, converting `Foo<Foo<T>>` to `Foo<T>`
    /// or from `Foo<Bar<T>>` to `Bar<T>` if suitable residual interconversion is implemented.
    fn flatten<V>(self) -> V
    where
        Self: Try<Output = V>,
        V: FromResidual<Self::Residual>,
    {
        self?
    }

    /// Calls a function with a reference to the contained value. Returns the original `Self`
    fn inspect<F>(self, f: F) -> Self
    where
        F: FnOnce(&Self::Output),
    {
        let val = self?;
        f(&val);
        Try::from_output(val)
    }

    /// Extracts the contained value `v` returning `Some(v)`, or `None` in the case of a Residual
    fn output<T>(self) -> Option<T>
    where
        Self: Try<Output = T>,
    {
        match self.branch() {
            ControlFlow::Continue(val) => Some(val),
            ControlFlow::Break(_) => None,
        }
    }

    /// Applys a function to the contained value converting `T` -> `U` then
    /// returns the canonical TryType for `Self` with Output `U`
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

    /// Converts from a `Foo<Bar<U>>` to a `Bar<Foo<U>>` where both `Foo` & `Bar` are `Try`.
    ///
    /// # Note
    ///
    /// - Return types are *canonical TryTypes*, for asymetrical cases this may not be `Bar` & `Foo`
    fn transpose<X, V, U>(self) -> X
    where
        // Foo<Bar<T>>
        Self: Try<Output = V>,
        // Bar<T>
        V: Try<Output = U>,
        // Bar<Foo<T>>
        X: Try + FromResidual<<Self::Output as Try>::Residual>,
        // Foo<T>
        X::Output: Try<Output = U> + FromResidual<Self::Residual>,
        // U *is* the canonical TryType for `Bar<Output=Foo<T>>`
        V::Residual: Residual<X::Output, TryType = X>,
        // U *wraps* the canonical TryType for `Foo<Output=T>`
        Self::Residual: Residual<U, TryType = X::Output>,
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

    /// Combines a `Foo<T>` with a `Bar<U>` into a `Foo<(T,U)>` where residual interconversion
    /// is available from `Bar->Foo`. Returns the *canonical TryType* based upon `Foo`.
    fn zip<X, V, U>(self, other: V) -> X
    where
        V: Try<Output = U>,
        X: Try<Output = (Self::Output, U)>
            + FromResidual<Self::Residual>
            + FromResidual<V::Residual>,
        Self::Residual: Residual<X::Output, TryType = X>,
    {
        let v1 = self?;
        let v2 = other?;
        Try::from_output((v1, v2))
    }

    /// Applies function `f` to the values inside `Foo<T>` & `Bar<U>` where residual interconversion
    /// is available from `Bar->Foo`. Returns the *canonical TryType* based upon `Foo`.
    ///
    /// TODO: #[unstable(feature = "option_zip", issue = "70086")]
    fn zip_with<X, V, F, G>(self, other: V, f: F) -> X
    where
        V: Try,
        X: Try<Output = G> + FromResidual<Self::Residual> + FromResidual<V::Residual>,
        Self::Residual: Residual<G, TryType = X>,
        F: FnOnce(Self::Output, V::Output) -> G,
    {
        let v1 = self?;
        let v2 = other?;
        Try::from_output(f(v1, v2))
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

    mod output {
        use super::*;

        #[test]
        fn ok() {
            let ok_5: Result<_, ()> = Ok(5);
            let stdlib = ok_5.ok();
            let custom = ok_5.output();
            assert_eq!(stdlib, custom);
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

        #[test]
        fn some_some_with() {
            let some_1 = Some(-1_i32);
            let some_2 = Some(2_u16);
            let stdlib = some_1.zip_with(some_2, |x, y| x + i32::from(y));
            let custom = Transform::zip_with(some_1, some_2, |x, y| x + i32::from(y));
            assert_eq!(stdlib, custom);
        }
    }
}
