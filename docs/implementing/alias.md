# Implementing Try

## Implementing other traits on Result (derive)

Type-alias-ing a Result is current idiom but this comes with one signifcant limitation. It is not possible to implement foreign traits on a type-alias. This is the case (wanting to impl Termination for the return type of main) that lead me to start down the rabbit-hole of “try” which lead to this book & the [`try_v2`](https://crates.io/crates/try_v2) crate. Check out the crate [`exit_safely`](https://crates.io/crates/exit_safely) to see the full use case.

### The problem

You can't do this:

```rust,ignore snippet
use std::process::Termination;

type Exit<E> = Result<(),E>;

impl<E> Termination for Exit<E> {
  ...
}
```

```text
error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate
 --> main.rs:5:1
  |
5 | impl<E> Termination for Exit<E> {
  | ^^^^^^^^^^^^^^^^^^^^^^^^-------
  |                         |
  |                         `std::result::Result` is not defined in the current crate
  |
  = note: impl doesn't have any local type before any uncovered type parameters
  = note: for more information see https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules
  = note: define and implement a trait or new type instead
```

### Derive(Try)

Manually implementing `Try` & friends for this comes with a chunk of boilerplate code and a few gotchas. Getting the same ergonomics that are available from Option, Try & Control-Flow adds even more boilerplate. As such the pay off was never there for me personally, until I was _forced_ to put a minimal implementation in place for [`exit_safely`](https://crates.io/crates/exit_safely). In true [pass-the-salt](https://xkcd.com/974/) style I went ahead and created the derive macros in [`try_v2`](https://crates.io/crates/try_v2).

```rust,ignore not_in_crate
#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]
use std::process::Termination;
use try_v2::Try;

#[derive(Debug, Try)]
#[must_use]
enum Exit<T: Termination> {
    Ok(T),
    Error(String),
    InvocationError(String),
}

impl<T: Termination> Termination for Exit<T> {
    fn report(self) -> std::process::ExitCode {
        todo!()
    }
}

fn main() {}
```

is the equivalent of manually implementing:

```rust
#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]
use std::process::Termination;

#[derive(Debug)]
#[must_use]
enum Exit<T: Termination> {
    Ok(T),
    Error(String),
    InvocationError(String),
}

// Recursive expansion of Try macro
// =================================

impl<T: Termination> std::ops::Try for Exit<T> {
    type Output = T;
    type Residual = Exit<!>;

    #[inline]
    fn from_output(output: Self::Output) -> Self {
        Self::Ok(output)
    }

    #[inline]
    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::Ok(v0) => std::ops::ControlFlow::Continue(v0),
            Self::Error(v0) => std::ops::ControlFlow::Break(Exit::Error(v0)),
            Self::InvocationError(v0) => std::ops::ControlFlow::Break(Exit::InvocationError(v0)),
        }
    }
}

impl<T: Termination> std::ops::FromResidual<Exit<!>> for Exit<T> {
    #[inline]
    #[track_caller]
    fn from_residual(residual: Exit<!>) -> Self {
        match residual {
            Exit::Error(v0) => Exit::Error(v0),
            Exit::InvocationError(v0) => Exit::InvocationError(v0),
        }
    }
}

impl<T: Termination> std::ops::Residual<T> for Exit<!> {
    type TryType = Exit<T>;
}

// =================================

impl<T: Termination> Termination for Exit<T> {
    fn report(self) -> std::process::ExitCode {
        todo!()
    }
}

fn main() {}
```
