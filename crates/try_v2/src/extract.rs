use std::ops::{ControlFlow, Try};

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
