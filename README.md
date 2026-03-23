# try_v2

A derive macro for [try_trait_v2](https://rust-lang.github.io/rfcs/3058-try-trait-v2.html)

Also enables auto-conversion from `Result<T, E> where E: Into::into(Self)`

## Requires

- `RUSTC_BOOTSTRAP = 1` (or nightly)
- `#![feature(never_type)]`
- `#![feature(try_trait_v2)]`

## Current Limitations on the annotated type

- must be an `enum`
- must have _one_ generic type
- the _first and only_ generic type must be the `Output` type (produced when not short circuiting)
- the output variant (does not short-circuit) must be the _first_ variant and store the output type as the _only unnamed_ field

## Example Usage

```rust
#![feature(never_type)]
#![feature(try_trait_v2)]
use try_v2::Try;

#[derive(Try)]
enum TestResult<T> {
    Ok(T),
    TestsFailed,
    OtherError(String)
}

fn run_tests() -> TestResult<()> {
    TestResult::OtherError("oops!".to_string())?; // <- Function short-circuits here ...
    TestResult::TestsFailed?;
    TestResult::Ok(())
}

assert!(matches!(run_tests(), TestResult::OtherError(msg) if msg == "oops!"));

struct MyError {}

impl<T> From<MyError> for TestResult<T> {
    fn from(err: MyError) -> Self {
        TestResult::TestsFailed
    }
}

fn run_more_tests() -> TestResult<()> {
    Err(MyError{})?; // <- Function short-circuits here & converts to a TestResult...
    TestResult::Ok(())
}

assert!(matches!(run_more_tests(), TestResult::TestsFailed));
```
