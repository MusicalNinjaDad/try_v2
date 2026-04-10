# Try_v2

 Provides a derive macro for `Try` & optionally `Try_ConvertResult` for interconversion with
 `std::result::Result` and `Try_Iterator` for iterating over `IntoIterator` and collecting from
 `FromIterator` analogous to how `Result` & `Option` do this.
 See ([try_trait_v2](https://rust-lang.github.io/rfcs/3058-try-trait-v2.html)) for more details
 of the underlying trait.

## Requires

- nightly
- `#![feature(never_type)]`
- `#![feature(try_trait_v2)]`
- `#![feature(try_trait_v2_residual)]`
- optionally:; `#![feature(iterator_try_collect)]` (if using Try_Iterator)

## Limitations on the annotated type

- must be an `enum`
- must have _at least one_ generic type
- the _first_ generic type must be the `Output` type (produced when not short circuiting)
- the output variant (does not short-circuit) must be the _first_ variant and store the output type as the _only unnamed_ field

See the [full documentation](https://docs.rs/try_v2/latest/try_v2/) for specifics on the generated code.

## Example Usage

```rust
#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]
use try_v2::{Try, Try_ConvertResult};

#[derive(Try, Try_ConvertResult)]
enum TestResult<T> {
    Ok(T),
    TestsFailed,
    OtherError(String)
}

// Basic short circuiting thanks to `#[derive(Try)]`
fn run_tests() -> TestResult<()> {
    TestResult::OtherError("oops!".to_string())?; // <- Function short-circuits here ...
    TestResult::TestsFailed?;
    TestResult::Ok(())
}

assert!(matches!(run_tests(), TestResult::OtherError(msg) if msg == "oops!"));


// Conversion from std::result::Result thanks to `#[derive(Try_ConvertResult)]`
struct TestFailure {}

impl<T> From<TestFailure> for TestResult<T> {
    fn from(err: TestFailure) -> Self {
        TestResult::TestsFailed
    }
}

fn run_more_tests() -> TestResult<()> {
    std::result::Result::Err(TestFailure{})?; // <- Function short-circuits here & converts to a TestResult...
    TestResult::Ok(())
}

assert!(matches!(run_more_tests(), TestResult::TestsFailed));
```

## MSRV

1.85.1, in case you are using a fixed version of nightly just to get access to specific unstable features.

## Currently untested (may work, may not ...)

- `where` clauses
- storing `Fn`s in variants
