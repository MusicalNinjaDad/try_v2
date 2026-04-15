# Thoughts on `try_trait_v2`

Let me start off by saying, I _love_ `trait Try` and really hope to see it accepted soon. I'm happy (and starting) to help with moving that forwards too.

## Emotionally

I like the trait for two reasons:

1. I don't like it when std can do stuff I can't, `Try` opens up the power and versatility of `?` to my own code.
1. I don't like writing extra code when it feels like I'm just working _around_ language constraints. `Try` lets me add `impl`s directly to a custom `Result` type or flatten nested contructs inside `Option`s & `Result`s

## My related crates

To date I've thought about using the trait multiple times but always found I would end up with more code than simply working around `Option` & `Result`. Then  I ran into a case where I **absolutely had to** add a trait to a `Result` - I wanted something to return from `fn main()` which gave me control over exit codes, ensured `Drop` was run properly _and_ didn't leave me with go-like error handling in `main` :feelsgood:

### exit_safely

[MusicalNinjaDad/exit_safely](https://github.com/MusicalNinjaDad/exit_safely) works with derived Try implementations via [MusicalNinjaDad/try_v2](https://github.com/MusicalNinjaDad/try_v2) to solve the problem of returning from main with Drop and control over exit codes.

### proc_macro2_diagnostic

[MusicalNinjaDad/proc_macro2_diagnostic](https://github.com/MusicalNinjaDad/proc_macro2_diagnostic) brings `?` to compiler diagnostics for proc macros.

### try_v2

[MusicalNinjaDad/try_v2](https://github.com/MusicalNinjaDad/try_v2) provides a set of derive macros to make `Try` more accessible. (See below for details)

## Criticism: complexity

After working with the trait in various use cases, taking it apart to try (!) and derive a generic implementation and spending time reading RFCs, unstable books, comments in std source code, github issues, PRs, discord discussions, ... to my mind, the remaining complexity in `Try` is **inherent**. The implementation is as simple as possible to provide the power and flexibility required for the more meaningful use cases.

## The 3 traits + 1 type + 1 function

When talking about `Try` below, I will usually consider the following traits in one package:

- `trait Try` (`try_trait_v2`)
- `trait FromResidual` (`try_trait_v2`)
- `trait Residual` (`try_trait_v2_residual`)
- `type !` (`never_type`)
- `fn try_collect()` (`iterator_try_collect`)

### 2 more experimental features

As wierd as it may be from the naming `try_blocks`, `try_blocks_heterogeneous` are more separate from a usage point of view.

## Simple case

Std contains 3 types which impl Try for all cases: `Result<T, E>`, `Option<T>` & `ControlFlow<B, C>`. Many of the most obvious uses for Try involve `Result`-like or `Option`-like situations and it is usually possible, if a little verbose / annoying, to work with `Result` & `Option` to get the same result (!).

### Flattening nested types

```rust
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
```

Wouldn't it be nicer to be able to have everthing in one place, with meaningful names for when I do want to handle non-`Some(Ok(Some(_)))` values? This is much easier to create, read, and reason about.

```rust
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
```

### Flattening trait MyFunctionsExt

Type-alias-ing a `Result` or `Option` is current idiom but ...

Adding methods and functions to a Type alias requires a custom trait, which adds an annoying copy of all the signatures that you need to keep up to date. It also pushes an implementation detail into the downstream code, if for example the functionality shadows a std or well-known 3rd-party trait...

The alternative is a NewType, which again pushes an implementation detail into downstream code which now needs to use `MyType.0`. Even worse, this adds a subtle nudge for people to `impl Deref` on their NewType - which is specifically _not_ designed for this kind of ergonomic hack.

```rust
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
```

With try this becomes much nicer to implement, read and use:

```rust
use log::info;
use std::{fmt::Display, ops::Add};

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
```

### Boilerplate -> Derive

While the above is really easy to create, use and reason about, manually implementing Try for this comes with a chunk of boilerplate code and

- traits themselves
- all the nice functions that std lib have in common

### Gotchas -> Derive

- choice of residual
- interconversion with result, overlapping Into impls
- &! not infallible

### Std inconsistencies & niggles

#### Poll - documentation

#### ControlFlow B, C not C, B

## Complex cases

### struct with hidden inner

### Box vs Vec vs Option

### ? with sideeffects

- global state inherently evil
- diagnosticresult
- loggedresult
- async & channels (LastPage, Page, Err)
