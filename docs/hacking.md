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

## Boilerplate & Gotchas -> Derive

While the above is really easy to create, use and reason about, manually implementing Try for this comes with a chunk of boilerplate code and a few gotchas. Getting the same ergonomics that are available from Option, Try & Control-Flow adds even more boilerplate. As such the pay off was never there for me personally, until I was _forced_ to put a minimal implementation in place for [exit_safely](https://crates.io/crates/exit_safely). In true [pass-the-salt](https://xkcd.com/974/) style I went ahead and created the derive macros in [try_v2](https://crates.io/crates/try_v2).

Note: I'd be happy to pass some (or all) of these into std, while leaving the "nice-to-haves" in a separate crate, if that would be valuable.

### Derivable case

The simple case described above is derivable (as in the examples) and is probably the most (numerically) common expected usage of Try. To be able to guarantee the `Foo<!>` pattern for the `Residual` and algorithmically generate arms for `branch()`, `from_residual()` etc. the macro enforces a few invariants on the annotated type:

- must be an `enum`
- must have _at least one_ generic type
- the _first_ generic type must be the `Output` type (produced when not short-circuiting)
- the output variant (does not short-circuit) must be the _first_ variant and store the output type as the _only unnamed_ field
- no other variant can store the Output type (TODO #72 add a nice error message)

While technically, the generic ordering requirement could be relaxed with slightly more complex logic, it is [deliberately tight](https://en.wikipedia.org/wiki/Poka-yoke) - to avoid accidental and hard to spot mistakes caused by switching generics.

### Derivable code

For the following case

```rust
#[derive(Try, Try_ConvertResult)]
enum TestResult<T, E> {
    Ok(T),
    TestsFailed,
    OtherError(E)
}
```

#### Macro `Try`: derives `Try`, `FromResidual` and `Residual`

will result in code of the shape:

```rust
impl<T,E> Try for TestResult<T, E> {
    type Output = T;
    type Residual = TestResult<!,E>;

    fn from_output(output: T) -> Self {
        Self::Ok(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        Self::Ok(t) => Continue(t),
        ... each failing variant => Break(failing variant) ...   
    }
}

impl<T, E> FromResidual<TestResult<!,E>> for TestResult<T, E> {
    fn from_residual(residual: TestResult<!,E>) -> Self {
        match residual {
            ... each failing variant => itself ...              
        }
    }
}

impl<T, E> Residual<T> for TestResult<!, E> {
    type TryType = TestResult<T, E>;
}
```

#### Macro `Try_ConvertResult`: derives bidirection `FromResidual` with `Result`

will generate

```rust

impl<T, E, RE> FromResidual<Result<Infallible, RE>> for TestResult<T, E>
where
    RE: Into<TestResult<!,E>>

... which calls Result::Err(e) => e.into(), ...
```

and

```rust
impl<E, RT, RE> FromResidual<TestResult<!,E>> for Result<RT, RE>
where
    RE: From<TestResult<!,E>>

... which calls Result::Err(residual.into()) ...
```

Why require `From/Into Foo<!>` and not `Foo<_>`? 2 reasons:

1. Otherwise you cannot create a non-conflicting implementation to allow for functions returning `Result<T, MyTry<!>>` to be ?-ed in functions returning `MyTry<U>`
2. It stops accidentally returning a `Result::Err(TestResult::Ok)` ([Poka-Yoke](https://en.wikipedia.org/wiki/Poka-yoke) again). If you actually want this ... don't derive as you probably need specific logic to handle this edge.

Effectively that allows using your type in any trait function where a `Result` is expected. Here's the `TryFrom` example from the integration tests. The subtle point to note: `let n = Even::try_from(num)?;` uses `?` to provide an `Even`, in a function that aims to return `Eightball<String>`, not `Eightball<Even>`

```rust
#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2::{Try, Try_ConvertResult};

#[derive(Try, Try_ConvertResult)]
#[must_use]
enum Eightball<Y> {
    Yes(Y),
    No,
}

struct Even(i32);

impl TryFrom<i32> for Even {
    type Error = Eightball<!>;

    fn try_from(num: i32) -> Result<Even, Eightball<!>> {
        if num % 2 == 0 {
            Result::Ok(Even(num))
        } else {
            Result::Err(Eightball::No)
        }
    }
}

fn even_string(num: i32) -> Eightball<String> {
    let n = Even::try_from(num)?;
    let s = format!("{}", n.0);
    Eightball::Yes(s)
}

assert!(matches!(even_string(2), Eightball::Yes(s) if s == "2"));
assert!(matches!(even_string(1), Eightball::No));
```

#### Macro `Try_Iterator`: derives `IntoIterator` and `FromIterator` analog to `Result` & `Option`

The stdlib implementations are almost identical. I took a lazy approach and have leveraged `std::option::IntoIter` to allow:

```rust
let tests: Vec<TestResult<i32, &'static str>> = vec![Ok(1), TestsFailed, Ok(2), OtherError("something wierd"), Ok(3), Ok(4)];

let first_results: TestResult<Vec<i32>, &'static str> = tests.into_iter().collect();
assert!(matches!(first_results, TestsFailed));

let mut test: TestResult<i32, &'static str> = Ok(4);
let borrowed_result: &i32 = test.iter().next().unwrap();
assert_eq!(borrowed_result, &4);
match test.iter_mut().next() {
    Some(v) => *v = 5,
    None => {},
}
assert!(matches!(test, TestResult::Ok(v) if v == 5));
let result = test.into_iter().next();
assert_eq!(result, Some(5));
```

#### Macro `Try_Methods` (WIP): derives `unwrap()`

`Option` & `Result` have a large set of sematically overlapping ergonomic methods for:

- Querying the variant
- Adapters for working with references (only `Option`)
- Extracting contained values
- Transforming contained values
- Boolean operators

Current task in progress is to derive equivalent methods named according to the enum variants. Right now, I have unwrap, goal is `is_testfailed()`, `expect()`, `othererror_or_else()`, `map_othererror` etc.

### Gotcha!s -> Derive

There are a few "gotcha!s" with even the simple implementation which can be easily avoided.

- Interconversion with Result, overlapping Into impls. This one bit me in the ass when using my own macros - while it may feel slightly awkward to require conversion to & from Result<_, MyResidual> anything else can trip you up later and be a pig to work out why (See [PR #50: fix Result Me bang (e.g. in TryFrom)](https://github.com/MusicalNinjaDad/try_v2/pull/50)).
- When working with references the compiler does not recognise `Foo::Ok(&!)` as an impossible variant and requires a match arm. It is all too tempting to use `unreachable!()` here - but safer to rely on the compiler either via `Ok(&t) => match {}` (safest) or `Ok(t) => *t` (slightly less safe)

### Std inconsistencies & niggles

A few things I noticed in std niggled me slightly

#### No clippy lint must_use_try

`Result` & `ControlFlow` are marked `#[must_use]` for good reason. `Option` is not, but possibly should be. I've added a compiler warning in the derive macro if the type is not `#[must_use]` but this cannot be silenced (yet, todo) and is not _really_ the right approach.

It would be a very valuable clippy lint to check that types which implement `Try` are labelled as `#[must_use]`. This would emit the warning when the user expects it - during linting - and can be silenced with an `#[allow(...)]`.

#### Poll - documentation

While `Option` & [`Result`](https://doc.rust-lang.org/std/result/index.html#the-question-mark-operator-) nicely document using `?`, [Poll](https://doc.rust-lang.org/std/task/enum.Poll.html) does not. I'd consider it really valuable to understand why the two specific implementations were chosen and how they are intended to be used:

```rust
impl<T, E, F> FromResidual<Result<Infallible, E>> for Poll<Option<Result<T, F>>>
where
    F: From<E>,
```

and

```rust
impl<T, E, F> FromResidual<Result<Infallible, E>> for Poll<Result<T, F>>
where
    F: From<E>,
```

#### ControlFlow<B, C> not ControlFlow<C, B>

`Result` and `Option` both lead with the generic for the Output type, ControlFlow does not. Given that the variants are ordered `Continue`, `Break` I find the alphabetical generics to be a regular source of "oops!"

## Complex cases

I've already run into 3 cases where I was not able to derive `Try`. I find two of them to be fine - they are cases where I want direct control over the mechanics.

### Struct with hidden inner

In [proc_macro2_diagnostic](https://crates.io/crates/proc_macro2_diagnostic) I chose to hide the enum behind an opaque struct. Primarily, I wanted to keep the specifics of the stored type as an implementation detail and find a pub enum which cannot be deconstructed or directly constructed to be "nasty". Secondly, I wanted to keep the variants of the enum as an implementation detail, allowing me to adjust them later.

This cost me some extra code but implementing `Try` etc. on a `struct` works perfectly fine.

### ? with side-effects

Let me start by offering an unrequested opinion: global state is inherently evil, hidden side-effects are inherently evil and usually rely on global state.

And yet ... also in [proc_macro2_diagnostic](https://crates.io/crates/proc_macro2_diagnostic) I have `?` with side-effects :flushed:. Top-level compiler diagnostics (on nightly) are not all errors, a custom `Try` implementation allowed both fatal errors & non-fatal warnings:

```rust
/// Result-like type which can represent a valid return value, an error or a warning accompanying
/// a valid return value. Warnings will be emitted upon `?`, allowing your code to continue with
/// the valid value. Errors will short-circuit upon `?` and be emitted upon final conversion to a
/// [proc_macro::TokenStream]
/// ...
pub struct DiagnosticResult<T> {
    inner: DiagnosticResult_<T>,
}
```

I'd consider the pattern both inherently dangerous and invaluable in select cases:

- `LoggedResult` (near the top of my todo list)

    ```rust
    /// Calling `?`:
    ///   - Ok(t) -> provides `t`;
    ///   - NonFatal(t, record) -> emits `record` to the logger & provides `t`
    ///   - Fatal(e) -> passes the error up the chain, without emitting anything.
    pub enum LoggedResult<T, E> {
        Ok(T),
        NonFatal(T, log::Record),
        Fatal(E)
    }
    ```

- Async cases, not so easily lib-ified, e.g. for handling paged responses to a query:

  ```rust
    /// Calling ?:
    ///   - LastPage(data) -> provides `data`;
    ///   - Page(data, next_page_uri) -> sends `next_page_uri` to page_handler channel & provides `data`
    ///   - Err(e) -> passes the error up the chain.
    struct PagedResponse<T, E> {
        handler: async_channel::Sender,
        payload: Payload<T, E>
    }

    enum Payload<T, E> {
        LastPage(T),
        Page(T, http::uri::Uri),
        Err(E),
    }
  ```

### Box vs Vec vs Option

This one I find more annoying. The simple case is safe to derive for outputs `T` and `&'t T` but not for `Box<T>`, `Vec<T>` etc. The compiler currently does not identify anything other than a pure `!` (or `Infallible` etc.) as impossible when checking match arms. For the purpose of match-arm completeness we currently need to write:

```rust
enum ValidatedBox<T> {
    ValidValue(Box<T>),
    InvalidValue,
}

use ValidatedBox::{InvalidValue, ValidValue};

let x: ValidatedBox<!> = InvalidValue;
let mut y = 0;

y += match x {
    InvalidValue => 1,
    ValidValue(_) => unreachable!("no way to construct a Box<!>"),
};

y += match x {
    InvalidValue => 1,
    ValidValue(b) => match *b {},
};

assert_eq!(y, 2);
```

which requires either manually stating that code is unreachable, not something I want to do in derived code, or knowing the specifics of the wrapper used and how to convert it to the inner type (not possible in derived code).

I can understand the troubles in differentiating `Box<!>`, `Vec<std::convert::Infallible>`, `Result<!,CustomZeroVariantEnum>` (all are verifiably impossible to construct) from `Option<!>` (can be `None`)! This is something that would be a valuable, and non-trivial, improvement to the compiler to improve ergonomics as more people begin to use `Try` and therefore `!`
