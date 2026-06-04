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
    use super::*;

    #[test]
    fn some_asref() {
        let some_5 = &Some(5);
        let stdlib = some_5.as_ref();
        let custom = AsRef::as_ref(some_5);
        assert_eq!(stdlib, custom);
    }
}
