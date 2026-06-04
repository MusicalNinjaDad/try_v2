use std::ops::{FromResidual, Residual, Try};

/// Adapters for working with references
pub trait AsRef<'s>: Try {
    /// Converts from &TryType<T> to Canonical<&T>
    fn as_ref<X>(&'s self) -> X
    where
        X: Try<Output = &'s Self::Output> + FromResidual<Self::Residual>,
        Self::Residual: Residual<&'s Self::Output, TryType = X>;

    /// Converts from &mut TryType<T> to Canonical<&mut T>
    fn as_mut<X>(&'s mut self) -> X
    where
        X: Try<Output = &'s mut Self::Output> + FromResidual<Self::Residual>,
        Self::Residual: Residual<&'s mut Self::Output, TryType = X>;
}

impl<'s, T> AsRef<'s> for Option<T>
where
    Self: 's,
    Self::Residual: Residual<&'s T, TryType = Option<&'s T>>,
{
    fn as_ref<X>(&'s self) -> X
    where
        X: Try<Output = &'s Self::Output> + FromResidual<Self::Residual>,
        Self::Residual: Residual<&'s Self::Output>,
    {
        Try::from_output(self.as_ref()?) // try bikeshed X { self.as_ref()? }
    }

    fn as_mut<X>(&'s mut self) -> X
    where
        X: Try<Output = &'s mut Self::Output> + FromResidual<Self::Residual>,
        Self::Residual: Residual<&'s mut Self::Output, TryType = X>,
    {
        Try::from_output(self.as_mut()?)
    }
}

#[cfg(test)]
mod tests {
    use try_v2_derive::Try;

    use super::*;

    #[derive(Try)]
    #[must_use]
    #[expect(dead_code)]
    enum YesNo<T, N> {
        Yes(T),
        No(N),
    }

    impl<'s, T, N> AsRef<'s> for YesNo<T, N> {
        fn as_ref<X>(&'s self) -> X
        where
            X: Try<Output = &'s Self::Output> + FromResidual<Self::Residual>,
            Self::Residual: Residual<&'s Self::Output, TryType = X>,
        {
            match *self {
                YesNo::Yes(ref y) => Try::from_output(y),
                YesNo::No(_) => todo!(),
            }
        }

        fn as_mut<X>(&'s mut self) -> X
        where
            X: Try<Output = &'s mut Self::Output> + FromResidual<Self::Residual>,
            Self::Residual: Residual<&'s mut Self::Output, TryType = X>,
        {
            todo!()
        }
    }

    #[test]
    fn some_asref() {
        let some_5 = &Some(5);
        let stdlib = some_5.as_ref();
        let custom = AsRef::as_ref(some_5);
        assert_eq!(stdlib, custom);
    }
}
