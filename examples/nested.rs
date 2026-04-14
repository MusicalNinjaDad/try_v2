#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]
#![allow(dead_code, clippy::disallowed_names)]

use std::{error::Error, io};

/// I _might_ have found somewhere that could contain duplicate info
///     Identifying duplicates _might_ have caused an error
///         And the answer _might_ be "no overlap"
type DuplicateData<T> = Option<Result<Option<T>, ValidErrors>>;

enum ValidErrors {
    Parsing(Box<dyn Error>),
    IO(Box<io::Error>),
}

// There is no good way to unpack this for use, or pass it "up the chain" for handling without
// repeated let-if-let-else-return directly in the code each time :(

fn main() {
    fn process(foo: DuplicateData<i32>) -> DuplicateData<i32> {
        let bar = if let Some(Ok(Some(value))) = foo {
            value
        } else {
            return foo;
        };
        let baz = bar + 1;
        Some(Ok(Some(baz)))
    }
    let foo: DuplicateData<i32> = Some(Ok(Some(5)));
    assert!(matches!(process(foo), Some(Ok(Some(6)))));
}

mod with_try {
    use std::{error::Error, io};

    use try_v2::Try;

    use DuplicateData::Duplicate;

    #[derive(Debug, Try)]
    #[must_use]
    enum DuplicateData<T> {
        Duplicate(T),
        NoCandidate,
        NoDuplicates,
        ParsingError(Box<dyn Error>),
        IOError(Box<io::Error>),
    }

    fn main() {
        fn process(foo: DuplicateData<i32>) -> DuplicateData<i32> {
            let baz = foo? + 1;
            Duplicate(baz)
        }
        let foo: DuplicateData<i32> = Duplicate(5);
        assert!(matches!(process(foo), Duplicate(6)));
    }
}
