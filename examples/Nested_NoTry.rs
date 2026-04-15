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
// repeated let Some(Ok(Some()))-else-return directly in the code each time :(

fn main() {
    fn process(foo: DuplicateData<i32>) -> DuplicateData<i32> {
        // This looks very much like:
        // bar, err := foo
        // if err != nil {
        //      return err
        // }
        let Some(Ok(Some(bar))) = foo else {
            return foo;
        };
        
        let baz = bar + 1;
        Some(Ok(Some(baz)))
    }
    let foo: DuplicateData<i32> = Some(Ok(Some(5)));
    assert!(matches!(process(foo), Some(Ok(Some(6)))));
}
