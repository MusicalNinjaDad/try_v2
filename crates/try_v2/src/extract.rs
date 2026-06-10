use std::{
    fmt::Debug,
    ops::{ControlFlow, Try},
};

/// Methods for extracting the wrapped value
pub trait Extract<T>
where
    Self: Try<Output = T> + Sized,
{
    /// Extracts the contained value `v` returning `Some(v)`, or `None` in the case of a Residual
    ///
    /// Think of this as the non-panicking version of `unwrap()`
    fn output(self) -> Option<T> {
        match self.branch() {
            ControlFlow::Continue(val) => Some(val),
            ControlFlow::Break(_) => None,
        }
    }

    /// Return the contained value or panic with a generic message.
    ///
    /// In general you should prefer `expect()` for panic situations or `output()` for non-panic
    ///
    /// # Note to implementors
    /// - the provided implementation uses [`Extract::expect`]. Implementors should prefer
    ///   customising `extract` to directly customising `unwrap`.
    fn unwrap(self) -> T
    where
        Self::Residual: Debug,
    {
        self.expect("called `unwrap()` on a residual:")
    }

    /// Return the contained value or panic with a custom message.
    ///
    /// In general you should prefer `output()` or `?` which do not panic.
    fn expect(self, msg: &str) -> T
    where
        Self::Residual: Debug,
    {
        match self.branch() {
            ControlFlow::Continue(v) => v,
            #[cfg(not(panic = "immediate-abort"))]
            ControlFlow::Break(r) => panic!("{msg}: {r:?}"),
            #[cfg(panic = "immediate-abort")]
            ControlFlow::Break(_) => panic!(),
        }
    }

    /// Return the contained value or the given default.
    ///
    /// Arguments passed to unwrap_or are eagerly evaluated; if you are passing the result of a
    /// function call, it is recommended to use unwrap_or_else, which is lazily evaluated.
    ///
    /// # Note to implementors
    /// - the provided implementation uses [`Extract::unwrap_or_else`]. Implementors should prefer
    ///   customising `unwrap_or_else` to directly customising `unwrap_or`.
    fn unwrap_or(self, default: T) -> T {
        self.unwrap_or_else(|| default)
    }

    /// Return the contained value or a default. Requires `T` to implement `Default`
    ///
    /// # Note to implementors
    /// - the provided implementation uses [`Extract::unwrap_or_else`]. Implementors should prefer
    ///   customising `unwrap_or_else` to directly customising `unwrap_or_default`.
    fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        self.unwrap_or_else(Default::default)
    }

    /// Return the contained value or a value computed from a closure.
    fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        match self.branch() {
            ControlFlow::Continue(v) => v,
            ControlFlow::Break(_) => f(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl<T> Extract<T> for Option<T> {}
    impl<T, E> Extract<T> for Result<T, E> {}

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
}
