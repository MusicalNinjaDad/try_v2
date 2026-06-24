use std::ops::{ControlFlow, FromResidual, Residual, Try};

impl<T> Transform<T> for Option<T> {}
impl<T, E> Transform<T> for Result<T, E> {}
impl<B, C> Transform<C> for ControlFlow<B, C> {}

/// Methods for transforming TryTypes. Inspired by the methods provided on `Option` & `Result`
///
/// ## Note
///
/// - TryTypes are recommended to directly implement specific `map_xxx` methods for *Residual* cases
///   with suitable where this makes sense. `map_residual()` provides for the cases where generic
///   naming is acceptable, and/or for implementations generic over `Try + Transform`
/// - Methods which act on the contained value will extract a value of type `Output` and return the
///   *canonical TryType* for the new Output. This is identifiable by the generic type `X` in the
///   method signature. This is usually the expected behaviour but can lead to a different value
///   type or resulting TryType where `Try` is not implemented symmetrically.
/// - Generic type conventions used in signatures (in standard order):
///     - `X` the *canonical TryType* returned
///     - `Y` the other TryType
///     - `T` the `Output` type for `Self`
///     - `U` the other `Output` type
///     - `F` a function/closure passed as a parameter
///     - `G` the return type of `F`
///     - `R` *never used* to avoid confusion with "Return" vs. "Residual".
pub trait Transform<T>
where
    Self: Try<Output = T> + Sized,
{
    /// Removes one level of nesting, converting `Foo<Foo<U>>` to `Foo<U>`
    /// or from `Foo<Bar<U>>` to `Bar<U>` if suitable residual inter-conversion is implemented.
    fn flatten(self) -> T
    where
        T: FromResidual<Self::Residual>,
    {
        self?
    }

    /// Calls a function with a reference to the contained value in the output case.
    /// Returns the original `Self`
    fn inspect<F>(self, f: F) -> Self
    where
        F: FnOnce(&T),
    {
        let val = self?;
        f(&val);
        Try::from_output(val)
    }

    /// Applies a function to the contained value (in the ouput case) converting `T` -> `U`,
    /// leaving residual cases untouched, then returns the canonical TryType for `Self` with
    /// Output `U`.
    fn map<X, U, F>(self, f: F) -> X
    where
        F: FnOnce(T) -> U,
        X: Try<Output = U> + FromResidual<Self::Residual>,
        Self::Residual: Residual<U, TryType = X>,
    {
        let val = self?;
        let mapped = f(val);
        Try::from_output(mapped)
    }

    /// Applies a function to the contained value (in the output case) converting `T` -> `U`,
    /// or returns the given `default` value.
    ///
    /// # Note
    /// - `default` is eagerly evaluated, prefer [Transform::map_or_else] if passing
    ///   the result of a function call.
    ///
    /// # Note to implementors
    /// - the provided implementation uses [`Transform::map_or_else`]. Implementors should prefer
    ///   customising `map_or_else` to directly customising `map_or`.
    fn map_or<U, F>(self, default: U, f: F) -> U
    where
        F: FnOnce(T) -> U,
    {
        self.map_or_else(|| default, f)
    }
    /// Applies a function to the contained value (in the output case) converting `T` -> `U`,
    /// or returns the result of `default()`.
    ///
    /// # Note
    /// - the closure `default` is lazily evaluated, prefer [Transform::map_or] if
    ///   passing a simple value.
    fn map_or_else<U, D, F>(self, default: D, f: F) -> U
    where
        D: FnOnce() -> U,
        F: FnOnce(T) -> U,
    {
        match self.branch() {
            ControlFlow::Continue(val) => f(val),
            ControlFlow::Break(_) => default(),
        }
    }

    /// Applies a function to the residual, leaving output values untouched, then returns the
    /// canonical TryType for the residual returned by the function (and Output `T`).
    ///
    /// # Note:
    /// - `f` takes the Residual and returns a Residual, not any values potentially wrapped by the
    ///   Residual. This is different from the specialised `Result::map_err()` which works on the
    ///   wrapped error value.
    ///
    /// ```
    /// # use try_v2_traits::Transform;
    /// let err_5: Result<String, i32> = Err(5);
    ///
    /// // Err(e).map_err() operates directly on e
    /// let stdlib = err_5.clone().map_err(|e| format!("{e}"));
    ///
    /// // Err(e).map_residual() operates on Err(e)
    /// // With only one residual variant direct destructuring is possible
    /// let custom = err_5.map_residual(|Err(e)| Err(format!("{e}")));
    ///
    /// assert_eq!(stdlib, custom);
    /// ```
    ///
    /// - This allows for interconversion between TryTypes if `f` returns a different residual:
    ///
    /// ```
    /// # use try_v2_traits::Transform;
    /// # use std::ops::ControlFlow;
    /// let err_5: Result<String, i32> = Err(5);
    /// let custom = err_5.map_residual(|Err(e)| ControlFlow::Break(e));
    /// assert_eq!(custom, ControlFlow::Break(5))
    /// ```
    ///
    /// ```
    /// # use try_v2_traits::Transform;
    /// // This is effectively the equivalent of Option::ok_or_else()
    /// let none: Option<i32> = None;
    /// let stdlib = none.ok_or_else(|| 5);
    /// let custom = none.map_residual(|_| Err(5));
    /// assert_eq!(stdlib, custom);
    /// ```
    fn map_residual<F, X, G>(self, f: F) -> X
    where
        F: FnOnce(Self::Residual) -> G,
        G: Residual<T, TryType = X>,
        X: Try<Output = T> + FromResidual<G>,
    {
        match self.branch() {
            ControlFlow::Continue(val) => X::from_output(val),
            ControlFlow::Break(residual) => X::from_residual(f(residual)),
        }
    }

    /// Converts from a `Foo<Bar<U>>` to a `Bar<Foo<U>>` where both `Foo` & `Bar` are `Try`.
    ///
    /// # Note
    /// - Return types are *canonical TryTypes*, for asymmetrical cases this may not be `Bar` & `Foo`
    fn transpose<X>(self) -> X
    where
        // Bar<T>
        T: Try,
        // Bar<Foo<T>>
        X: Try + FromResidual<T::Residual>,
        // Foo<T>: Try<Output = T>         + FromResidual<Foo<!>>
        X::Output: Try<Output = T::Output> + FromResidual<Self::Residual>,
        // X *is* the canonical TryType for `Bar<Output=Foo<T>>`
        T::Residual: Residual<X::Output, TryType = X>,
        // X *wraps* the canonical TryType for `Foo<Output=T>`
        Self::Residual: Residual<T::Output, TryType = X::Output>,
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

    /// `foo.zip(bar)` combines a `Foo<T>` with a `Bar<U>` into a `Foo<(T,U)>` where residual
    /// interconversion is available from `Bar->Foo`. Returns the *canonical TryType* based upon
    /// `Foo`. Returns a residual if either `Foo` or `Bar` are residuals.
    ///
    /// # Note to implementors
    /// - the provided implementation uses [`Transform::zip_with`]. Implementors should prefer
    ///   customising `zip_with` to directly customising `zip`.
    fn zip<X, Y>(self, other: Y) -> X
    where
        Y: Try,
        X: Try<Output = (T, Y::Output)> + FromResidual<Self::Residual> + FromResidual<Y::Residual>,
        Self::Residual: Residual<X::Output, TryType = X>,
    {
        self.zip_with(other, |t, u| (t, u))
    }

    /// `foo.zip_with(bar, do_stuff)` applies `do_stuff(foo?, bar?)` to the combined values inside
    /// `Foo<T>` & `Bar<U>` where residual interconversion is available from `Bar->Foo`.
    /// Returns the *canonical TryType* based upon `Foo`.
    ///
    /// # Note
    /// - this is equivalent to [unstable feature `option_zip`](https://github.com/rust-lang/rust/issues/70086)
    fn zip_with<X, Y, F, G>(self, other: Y, f: F) -> X
    where
        Y: Try,
        F: FnOnce(T, Y::Output) -> G,
        X: Try<Output = G> + FromResidual<Self::Residual> + FromResidual<Y::Residual>,
        Self::Residual: Residual<G, TryType = X>,
    {
        let v1 = self?;
        let v2 = other?;
        Try::from_output(f(v1, v2))
    }

    /// `foo.and(bar)` returns `bar` if `foo` is the output case, otherwise returns `foo`.
    ///
    /// # Note
    /// - Unlike `Result::and()` this will also allow `Result<T,E>.and(Result<T,F>)` where `F: From(E)`
    ///
    /// # Note to implementors
    /// - the provided implementation uses [`Transform::and_then`]. Implementors should prefer
    ///   customising `and_then` to directly customising `and`.
    fn and<Y>(self, other: Y) -> Y
    where
        Y: Try<Output = T> + FromResidual<Self::Residual>,
    {
        self.and_then(|_| other)
    }

    /// `foo.and_then(bar)` calls `bar()` if `foo` is the output case, otherwise returns `foo`.
    fn and_then<Y, F>(self, f: F) -> Y
    where
        Y: Try<Output = T> + FromResidual<Self::Residual>,
        F: FnOnce(T) -> Y,
    {
        f(self?)
    }

    /// `foo.or(bar)` will return a `Bar<T>` wrapping the contents of `foo` if `foo` is the output
    /// case or `bar` if `foo` is a residual.
    ///
    /// # Note
    /// - This will convert types - so `Ok(5).or(None) = Some(5)`
    ///
    /// # Note to implementors
    /// - the provided implementation uses [`Transform::or_else`]. Implementors should prefer
    ///   customising `or_else` to directly customising `or`.
    fn or<Y>(self, other: Y) -> Y
    where
        Y: Try<Output = T>,
    {
        self.or_else(|_| other)
    }

    /// `foo.or_else(bar)` where `bar` is a function returning a `Bar<T>` will return a `Bar<T>`
    /// wrapping the contents of `foo` if `foo` is the output case or call `bar(foo)` if `foo` is
    /// a residual.
    ///
    /// # Note
    /// -  The closure receives `Self::Residual` (unlike `Option::or_else` & `Result::or_else` which
    ///    are specialised to receive `()` & `E` respectively)
    fn or_else<Y, F>(self, f: F) -> Y
    where
        Y: Try<Output = T>,
        F: FnOnce(Self::Residual) -> Y,
    {
        match self.branch() {
            ControlFlow::Continue(v) => Try::from_output(v),
            ControlFlow::Break(r) => f(r),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod boolean {
        use super::*;

        #[test]
        fn and_some_some() {
            let some_5 = Some(5);
            let some_6 = Some(6);
            let stdlib = some_5.and(some_6);
            let custom = Transform::and(some_5, some_6);
            assert_eq!(stdlib, custom);
        }

        #[test]
        fn and_none_some() {
            let some_5 = None;
            let some_6 = Some(6);
            let stdlib = some_5.and(some_6);
            let custom = Transform::and(some_5, some_6);
            assert_eq!(stdlib, custom);
        }

        #[test]
        fn and_some_none() {
            let some_5 = Some(5);
            let some_6 = None;
            let stdlib = some_5.and(some_6);
            let custom = Transform::and(some_5, some_6);
            assert_eq!(stdlib, custom);
        }

        #[test]
        fn or_ok_ok() {
            let ok_5: Result<_, ()> = Ok(5);
            let ok_6: Result<_, i32> = Ok(6);
            let stdlib = ok_5.or(ok_6);
            let custom = Transform::or(ok_5, ok_6);
            assert_eq!(stdlib, custom);
        }

        // TODO document this well
        #[test]
        fn or_ok_some() {
            let ok_5: Result<_, ()> = Ok(5);
            let some_6 = Some(6);
            let custom = Transform::or(ok_5, some_6);
            assert_eq!(custom, Some(5));
        }
    }

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

        #[test]
        fn map_residual() {
            let err_5: Result<String, i32> = Err(5);
            let stdlib = err_5.clone().map_err(|e| format!("{e}"));
            let custom = err_5.map_residual(|Err(e)| Err(format!("{e}")));
            assert_eq!(stdlib, custom);
        }

        #[test]
        fn map_residual_conversion() {
            let err_5: Result<String, i32> = Err(5);
            let custom = err_5.map_residual(|Err(e)| ControlFlow::Break(e));
            assert_eq!(custom, ControlFlow::Break(5))
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

        #[test]
        // https://rust-lang.zulipchat.com/#narrow/channel/605325-t-lang.2Ftry/topic/Type.20inference.20is.20one-way/near/599375356
        fn reverse_infer() {
            let some_1 = Some(-1_i32);
            let some_2 = Some(2_u16);
            let custom: Option<()> = Transform::zip_with(some_1, some_2, |_, _| Default::default());
            assert_eq!(custom, Some(()));
        }
    }
}
