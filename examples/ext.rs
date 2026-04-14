#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]
#![allow(dead_code, clippy::disallowed_names)]

type Counter = Option<i32>;

/// While not so much extra work to double-specify everything as a trait & impl, it still
/// wastes keystrokes and
trait NumberExt {
    fn new() -> Self;
    fn inc(self) -> Self;
    // etc...
}

impl NumberExt for Counter {
    fn new() -> Self {
        Some(0)
    }

    fn inc(self) -> Self {
        Some(self? + 1)
    }
}

fn main() {
    let foo = Counter::new();
    assert_eq!(foo.inc().unwrap(), 1);
}

mod with_try {
    use std::ops::Add;

    use try_v2::Try;

    use Counter::Count;

    #[derive(Debug, Try)]
    #[must_use]
    enum Counter<N> {
        Count(N),
        Uninitialised,
    }

    impl Counter<i32> {
        fn new() -> Self {
            Count(0)
        }

        fn inc(self) -> Self {
            self + 1
        }
    }

    /// Not possible on a type alias, which leads to some people `impl Deref` to avoid
    /// peppering code with `.0` for a NewType or working around it in other ways.
    impl<N, M> Add<M> for Counter<N>
    where
        N: Add<M, Output = N>,
    {
        type Output = Self;

        fn add(self, rhs: M) -> Self::Output {
            Self::Count(self? + rhs)
        }
    }

    fn main() {
        let foo = Counter::new();
        assert_eq!(foo.inc().unwrap(), 1);
    }
}
