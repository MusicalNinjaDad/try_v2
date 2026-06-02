#![allow(clippy::disallowed_names)]

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
    assert!(matches!(foo.inc(2), Some(2)));
}
