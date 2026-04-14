#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]
#![allow(dead_code, clippy::disallowed_names)]

use log::info;

/// A Counter which logs each adjustment
type Counter<N> = Option<N>;

// While not so much extra work to double-specify everything as a trait & impl, it still
// wastes keystrokes and
trait NumberExt<N> {
    fn new() -> Self;
    fn inc(self, n: N) -> Self;
    // etc...
}

impl NumberExt<i32> for Counter<i32> {
    fn new() -> Self {
        info!("new counter initialised");
        Some(0)
    }

    // Not possible to impl std::ops::Add on a type alias, which leads to some people `impl Deref`
    // to avoid peppering code with `.0` for a NewType or working around it in other ways, like this.
    //
    // Both cases force consideration of the implementation details in the code which uses Counter.
    fn inc(self, n: i32) -> Self {
        info!("adding {n} to counter");
        let n = self? + n;
        info!("new value {n}");
        Some(n)
    }
}

fn main() {
    let foo = Counter::new();
    assert_eq!(foo.inc(2).unwrap(), 2);
}

mod with_try {
    use std::{fmt::Display, ops::Add};
    use log::info;

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
            info!("new counter initialised");
            Count(0)
        }
    }

    // More versatile implementation, better separating responsibilities:
    // implementation details are owned here, type specifics at usage site.
    impl<N, M> Add<M> for Counter<N>
    where
        N: Add<M, Output = N> + Display,
        M: Display,
    {
        type Output = Self;

        fn add(self, rhs: M) -> Self::Output {
            info!("adding {rhs} to counter");
            let n = self? + rhs;
            info!("new value {n}");
            Self::Count(n)
        }
    }

    fn main() {
        let foo = Counter::new();
        assert!(matches!(foo + 2, Count(n) if n ==2));
    }
}
