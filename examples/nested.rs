#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]
#![allow(dead_code, clippy::disallowed_names)]

use std::{error::Error, io};

/// I _might_ know whether I have an answer at all
///     Getting the answer _might_ have caused an error
///         And the answer _might_ be "nope"
type MaybeMaybe<T> = Option<Result<Option<T>, ValidErrors>>;

enum ValidErrors {
    Parsing(Box<dyn Error>),
    IO(Box<io::Error>),
}

// There is no good way to unpack this for use, or pass it "up the chain" for handling without
// repeated let-if-let-else-return directly in the code each time :(

fn main() {
    fn increment_foo(foo: MaybeMaybe<i32>) -> MaybeMaybe<i32> {
        let bar = if let Some(Ok(Some(value))) = foo {
            value
        } else {
            return foo;
        };
        let baz = bar + 1;
        Some(Ok(Some(baz)))
    }
    let foo: MaybeMaybe<i32> = Some(Ok(Some(5)));
    assert!(matches!(increment_foo(foo), Some(Ok(Some(6)))));
}

mod with_try {
    use std::{error::Error, io};

    use try_v2::Try;

    use MaybeMaybe::Ok;

    #[derive(Debug, Try)]
    #[must_use]
    enum MaybeMaybe<T> {
        Ok(T),
        NoAnswer,
        NoValue,
        ParsingError(Box<dyn Error>),
        IOError(Box<io::Error>),
    }

    fn main() {
        fn increment_foo(foo: MaybeMaybe<i32>) -> MaybeMaybe<i32> {
            let baz = foo? + 1;
            Ok(baz)
        }
        let foo: MaybeMaybe<i32> = Ok(5);
        assert!(matches!(increment_foo(foo), Ok(6)));
    }
}
