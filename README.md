# try_v2

A derive macro for [try_trait_v2](https://rust-lang.github.io/rfcs/3058-try-trait-v2.html)

## Requires

- `RUSTC_BOOTSTRAP = 1` (or nightly)
- `#![feature(never_type)]`
- `#![feature(try_trait_v2)]`

## Current Limitations on the annotated type

- must be an `enum`
- must have _at least one_ generic type
- the _first_ generic type must be the `Output` type (produced when not short circuiting)
- the output variant (does not short-circuit) must be the _first_ variant

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

assert!(matches!(run_tests(), TestResult::OtherError(msg) if msg == "oops!"))
```
