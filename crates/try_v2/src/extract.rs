use std::{
    fmt::Debug,
    ops::{ControlFlow, Try},
};

/// Methods for extracting the wrapped value
pub trait Extract
where
    Self: Try + Sized,
{
    /// Extracts the contained value `v` returning `Some(v)`, or `None` in the case of a Residual
    ///
    /// Think of this as the non-panicking version of `unwrap()`
    fn output<T>(self) -> Option<T>
    where
        Self: Try<Output = T>,
    {
        match self.branch() {
            ControlFlow::Continue(val) => Some(val),
            ControlFlow::Break(_) => None,
        }
    }

    /// Return the contained value or panic with a generic message.
    ///
    /// In general you should prefer `expect()` for panic situations or `output()` for non-panic
    fn unwrap<T>(self) -> T
    where
        Self: Try<Output = T>,
        Self::Residual: Debug,
    {
        match self.branch() {
            ControlFlow::Continue(v) => v,
            #[cfg(not(panic = "immediate-abort"))]
            ControlFlow::Break(r) => panic!("called `unwrap()` on a residual: {r:?}"),
            #[cfg(panic = "immediate-abort")]
            ControlFlow::Break(_) => panic!(),
        }
    }

    /// Return the contained value or panic with a custom message.
    ///
    /// In general you should prefer `output()` or `?` which do not panic.
    fn expect<T>(self, msg: &str) -> T
    where
        Self: Try<Output = T>,
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
    fn unwrap_or<T>(self, default: T) -> T
    where
        Self: Try<Output = T>,
    {
        match self.branch() {
            ControlFlow::Continue(v) => v,
            ControlFlow::Break(_) => default,
        }
    }

    /// Return the contained value or a default. Requires `T` to implement `Default`
    fn unwrap_or_default<T>(self) -> T
    where
        Self: Try<Output = T>,
        T: Default,
    {
        match self.branch() {
            ControlFlow::Continue(v) => v,
            ControlFlow::Break(_) => Default::default(),
        }
    }

    /// Return the contained value or a value computed from a closure.
    fn unwrap_or_else<T, F>(self, f: F) -> T
    where
        Self: Try<Output = T>,
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

    impl<T> Extract for Option<T> {}
    impl<T, E> Extract for Result<T, E> {}

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
