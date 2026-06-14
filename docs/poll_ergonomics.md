# Thoughts on Poll & overall try ergonomics

## Current `Poll` implementation

Poll's current implementations are heavily coupled to an expectation that `?` should always act on an inner Result:

```rust
impl<T, E> ops::Try for Poll<Result<T, E>> {
    type Output = Poll<T>;
    type Residual = Result<convert::Infallible, E>;
```

```rust
impl<T, E> ops::Try for Poll<Option<Result<T, E>>> {
    type Output = Poll<Option<T>>;
    type Residual = Result<convert::Infallible, E>;
```

A bit of archaelogy suggests that this is for historical reasons - the implementation dates from [a time before](https://doc.rust-lang.org/1.36.0/std/task/enum.Poll.html) `?` when `try!()` and [fallibility related only to errors](https://doc.rust-lang.org/1.36.0/std/ops/trait.Try.html) (and not even to `Option`s)

I find this very confusing in today's rust:

1. Consistency: I'd expect `let v: i32 = Poll::Ready<3>?` to work and `let v: i32 = Poll::Pending?` to early return `Poll::Pending`
2. Handling `None` from a `Poll<Option<Result<>>>` is significantly different than handling `Err` from a `Poll<Result<>>`, and even more confusingly the `?` acts on the innermost result.

### Implications for try_fold() etc

Currently, using `try_fold()` on an `impl Iterator<Item=Poll<Result<T,E>>>` requires some interesting gymnastics (so much that specifying the types seems like a good idea!)

```rust
use std::task::Poll;

fn main() {
    let polls = [
        Poll::Ready(Ok(0)),
        Poll::Pending,
        Poll::Ready(Err(2)),
        Poll::Ready(Ok(3)),
    ];
    let _ = polls.into_iter().try_fold(
        Poll::Ready(0),
        |total: Poll<i32>, n: Poll<Result<i32, i32>>| -> Result<Poll<i32>, i32> {
            let n: Poll<i32> = n?; // <- shorts on Poll::Ready(Err)
            let total: Poll<i32> = match n {
                Poll::Ready(n) => total.map(|prev| prev + n),
                Poll::Pending => Poll::Pending,
            };
            Ok(total)
        },
    );
}
```

A homogeneous implementation would allow for

```rust
use std::task::Poll;

fn main() {
    let polls = [
        Poll::Ready(Ok(0)),
        Poll::Pending,
        Poll::Ready(Err(2)),
        Poll::Ready(Ok(3)),
    ];
    let _ = polls
        .into_iter()
        .try_fold(0, |total, n| Ready(Ok(total + n??)));
```

### Wider implications for all future TryTypes

`scottmcm` on [try{}: What does homogeneity mean for Poll?
 #155368](https://github.com/rust-lang/rust/issues/155368)

> The idea of [Add homogeneous_try_blocks RFC #3721](https://github.com/rust-lang/rfcs/pull/3721) is that you get "the same" thing out of `try { x? }`.
>
> However, there's currently no requirement that that actually happen in either the trait nor the desugaring.
>
> As usual, `Poll` is the problem child for this: <https://doc.rust-lang.org/std/task/enum.Poll.html#impl-Try-for-Poll%3COption%3CResult%3CT,+E%3E%3E%3E>
>
> Some potential options:
>
> 1. Add a bound like `type Residual: Residual<Self::Output, TryType = Self>;` to the `Try` trait so that this can't happen any more.
>     - that insists on separately-coloured residuals.  This would require changing the residual types for `Poll` to be distinct, and to update the `FromResidual` implementations accordingly to continue to support stable things.
> 2. Give up and say "meh, whatever" and `try { my_poll? }` will give a `Result` or an `Option`.
> 3. Add a bound into the desugaring for `?` or `try {` somehow that would enforce that it's usable only on `Try` types with full homogeneity
>     - perhaps merge the branch+from_residual into one helper function that calls both but which has some extra `where` bounds

#### Things which are blocked by this

- Providing a standard implementation of `map` etc. difficult - as it would also act "unusually" on Poll, like `try_fold()` does.
- Allowing for `?`-chaining out-of-the-box (see below)

### What would homogeneous `Try for Poll` look like?

Changing the implementation of `Try for Poll` to be homogeneous would lead to:

```rust
impl<T> Try for Poll<T> {
    Output = T;
    Residual = Poll<!>;
    ...
}

impl<T> FromResidual<Poll<!>> for Poll<T> { ... }
impl<T, E, F: From<E>> ops::FromResidual<Result<convert::Infallible, E>> for Poll<Result<T, F>> { ... }
// Recommended but optional also ...
impl<T> ops::FromResidual<Option<convert::Infallible>> for Poll<Option<T>> { ... }

impl<T> Residual<T> for Poll<!> {
    TryType = Poll<T>
}
```

This would keep the current direct `FromResidual` interconversion, but require removal of the current

```rust
impl<T, E> ops::Try for Poll<Result<T, E>> {
    type Output = Poll<T>;
    type Residual = Result<convert::Infallible, E>;
    ...
}

impl<T, E> ops::Try for Poll<Option<Result<T, E>>> {
    type Output = Poll<Option<T>>;
    type Residual = Result<convert::Infallible, E>;
    ...
}

impl<T, E, F: From<E>> FromResidual<Result<!, E>> for Poll<Option<Result<T, F>>> { ... }
```

### Current usage & breakages

There are 3 patterns to using the current implementation, all of which can be found in the wild (below examples are all from `tokio`)

#### 1. `ready!(poll_foo(cx))?` in `fn -> Poll<Result<_,_>>` - Most common

This is by far the most common case. It relies only on `FromResidual<Result<!,E> for Poll<Result<T,F>>` and as such would not break.

Future uses could be made more ergonomic by simply writing `poll_foo(cx)??`.

[Example](https://github.com/tokio-rs/tokio/blob/778e9d97d91cff2c3036cc5663936d479edb30cb/tokio-util/src/udp/frame.rs#L169)

```rust
fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    ...
}

fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    ready!(self.poll_flush(cx))?; // <- Uses FromResidual<Result<!,E>> for Poll<Result<T,E>>
    Poll::Ready(Ok(()))
}
```

#### 2. `ready!(poll_foo(cx)?)` - Rare

This would break, *with a compiler error*.

In `fn -> Poll<Result<_,_>>` this is semantically identical to `ready!(poll_foo(cx))?` and would be fixed by simply moving the `?`.

In `fn -> Poll<Option<Result<_,_>>>` this would now short on `None` leaving the Result to be handled. Given the low readability of code using this pattern, which effectively handles the wrapped types out of order, the change would be a clear improvement to code quality and reduce future bug risk and maintenance costs. (See also pattern 3)

- `poll_foo(cx)???` if `Poll::Ready(None)` should return `Poll::Ready(None)`
- `poll_foo(cx)?.ready()??` if `Poll::Ready(None)` should return `Poll::Pending` (assuming `Option::ready()` analogous to `Option.ok_or()`)

[Example](https://github.com/tokio-rs/tokio/blob/778e9d97d91cff2c3036cc5663936d479edb30cb/tokio/src/io/util/take.rs#L116)

```rust
fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
    ...
    let buf = ready!(me.inner.poll_fill_buf(cx)?);
    ...
    Poll::Ready(Ok(&buf[..cap]))
}
```

becomes

```rust
fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
    ...
    let buf = ready!(me.inner.poll_fill_buf(cx))?; // or me.inner.poll_fill_buf(cx)??
    ...
    Poll::Ready(Ok(&buf[..cap]))
}
```

#### 3. `match poll_foo(cx)?` - Rare

This would break *with a compiler error*.

Updated code would be able to avoid the `match` entirely and rely on `map()`, `?`, etc. which would be more idiomatic overall.

[Example](https://github.com/tokio-rs/tokio/blob/778e9d97d91cff2c3036cc5663936d479edb30cb/tokio-util/src/udp/frame.rs#L115)

```rust
fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    if !self.flushed {
        match self.poll_flush(cx)? {
            Poll::Ready(()) => {}
            Poll::Pending => return Poll::Pending,
        }
    }

    Poll::Ready(Ok(()))
}
```

becomes

```rust
fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    if !self.flushed {
        self.poll_flush(cx)??;
    }

    Poll::Ready(Ok(()))
}
```

## Chaining `?`

Currently, chaining `?` relies on TryTypes providing specific implementations of `FromResidual`. To aid with the migration of code if Poll were made homogeneous, I suggested such implementations above.

When `Try` stabilises, we should expect it to be useful and therefore used. As such, we will see new TryTypes provided by crates outside stdlib and therefore more cases of `Foo<Bar<T>>` where `Foo` & `Bar` are both `Try`.

To help with user ergonomics I suggest allowing chaining `?` via one of 2 methods.

### Blanket `FromResidual`

If done now the following implementation in stdlib would provide `?`-chaining simply and easily via

```rust
impl<T,X,Y> FromResidual<Y::Residual> for X 
where 
X: Try<Output = Y>, // `Foo<Bar<T>>`
Y: Try<Output = T>, // `Bar<T>`
{
    fn from_residual(residual: Y::Residual) -> Self {
        Try::from_output(FromResidual::from_residual(residual))
    }
}
```

Right now, this would collide with `Poll` (see ~~diatribe~~ discussion above)

Doing this would further help with homogeneity as it would forbid non-homogeneous implementations by 3rd party types which would likely be a siginificant source of confusion for users.

### Specific desugaring

The compiler *could* specifically desugar multiple `?`s. But I find it hard to argue why this should happen if a stdlib implementation is possible.

## Additional methods for TryTypes - e.g. map()

Working with custom TryTypes has highlighted the value of the wide range of methods for transforming, extracting and working with the contained values. While TryTypes are generally very unique in their Residual, both type and semantics, they all provide a single `Output`. Generic versions of most of the functions common to `Option` & `Result` are possible in so far as they relate to the output case.

### Usages blocked by missing implementations

If such implementations are missing this blocks usage in two general cases

1. Using a `?` on a `Foo<T>` in a function that returns `Foo<U>` (without also using `try {...}` which is likely to remain experimental after `trait Try` is stabilised)
1. Being generic over `Try` without being able to rely on the existance of `map` & friends.

### Implementing the methods

Expecting authors of custom TryTypes to identify and correctly implement the required signatures individually is probably unfair.

Generic type conventions used in signatures (in standard order):

- `X` the *canonical* TryType returned
- `Y` the other TryType
- `T` the `Output` type for `Self`
- `U` the other `Output` type
- `F` a function/closure passed as a parameter
- `G` the return type of `F`
- `R` never used to avoid confusion with “Residual”.

[Signatures from `try_v2::Transform`](https://docs.rs/try_v2/latest/try_v2/trait.Transform.html)

```rust
pub trait Transform<T>: Sized + Try<Output = T> {
    // Provided methods
    fn flatten(self) -> T
       where T: FromResidual<Self::Residual> { ... }
    fn inspect<F>(self, f: F) -> Self
       where F: FnOnce(&T) { ... }
    fn map<X, U, F>(self, f: F) -> X
       where F: FnOnce(T) -> U,
             X: Try<Output = U> + FromResidual<Self::Residual>,
             Self::Residual: Residual<U, TryType = X> { ... }
    fn map_or<U, F>(self, default: U, f: F) -> U
       where F: FnOnce(T) -> U { ... }
    fn map_or_else<U, D, F>(self, default: D, f: F) -> U
       where D: FnOnce() -> U,
             F: FnOnce(T) -> U { ... }
    fn transpose<X>(self) -> X
       where T: Try,
             X: Try + FromResidual<T::Residual>,
             X::Output: Try<Output = T::Output> + FromResidual<Self::Residual>,
             T::Residual: Residual<X::Output, TryType = X>,
             Self::Residual: Residual<T::Output, TryType = X::Output> { ... }
    fn zip<X, Y>(self, other: Y) -> X
       where Y: Try,
             X: Try<Output = (T, Y::Output)> + FromResidual<Self::Residual> + FromResidual<Y::Residual>,
             Self::Residual: Residual<X::Output, TryType = X> { ... }
    fn zip_with<X, Y, F, G>(self, other: Y, f: F) -> X
       where Y: Try,
             F: FnOnce(T, Y::Output) -> G,
             X: Try<Output = G> + FromResidual<Self::Residual> + FromResidual<Y::Residual>,
             Self::Residual: Residual<G, TryType = X> { ... }
    fn and<Y>(self, other: Y) -> Y
       where Y: Try<Output = T> + FromResidual<Self::Residual> { ... }
    fn and_then<Y, F>(self, f: F) -> Y
       where Y: Try<Output = T> + FromResidual<Self::Residual>,
             F: FnOnce(T) -> Y { ... }
    fn or<Y>(self, other: Y) -> Y
       where Y: Try<Output = T> { ... }
    fn or_else<Y, F>(self, f: F) -> Y
       where Y: Try<Output = T>,
             F: FnOnce(Self::Residual) -> Y { ... }
}
```

[Signatures from `try_v2::Extract`](https://docs.rs/try_v2/latest/try_v2/trait.Extract.html)

```rust
pub trait Extract<T>: Sized + Try<Output = T> {
    // Provided methods
    fn output(self) -> Option<T> { ... }
    fn unwrap(self) -> T
       where Self::Residual: Debug { ... }
    fn expect(self, msg: &str) -> T
       where Self::Residual: Debug { ... }
    fn unwrap_or(self, default: T) -> T { ... }
    fn unwrap_or_default(self) -> T
       where T: Default { ... }
    fn unwrap_or_else<F>(self, f: F) -> T
       where F: FnOnce() -> T { ... }
}
```

Each of these does what you would expect, if you are used to working with `Option` & `Result` with the following caveats:

1. Returns are always *canonical* TryTypes - this is not an issue where homogeneity is expected (see `Poll`) and is no more of an issue than `try_fold` etc. in those cases.
1. `or` is more flexible than current implementations as it allows `Ok(5).or(None) == Some(5)`.

## Make TryFrom generic over Try

### Current workaround (hack) - use `Foo::Residual` as `Error`

Currently to use a custom TryType with TryFrom requires a workaround which feels more like a nasty hack.

```rust
#[derive(Try, Try_ConvertResult)]
#[must_use]
enum Eightball<Y, N> {
    Yes(Y),
    TryAgain,
    No(N),
}

// the derive provides the generally useful:
impl<N,T,E: From<Eightball<!,N>>> FromResidual<Eightball<!,N>> for Result<T,E> { ... }

// which allows for:
impl TryFrom<i32> for Even2 {
    type Error = Eightball<!, Odd>;

    fn try_from(num: i32) -> Result<Even2, Eightball<!, Odd>> {
        if num % 2 == 0 {
            Result::Ok(Even2(num))
        } else {
            Result::Err(Eightball::No(Odd(num)))
        }
    }
}
```

### Non-breaking change

A non-breaking change to TryFrom would allow it to return an arbitrary TryType, and leverage `FromResidual` implementations to provide the ability to call from a function returning another TryType, such as a `Result`.

```rust
pub trait TryFrom<T>: Sized {
    /// Must keep type, otherwise would be a breaking change
    type Error = !;
    /// The specific Try-type to return
    /// Defaults to Result, to make this a non-breaking change
    type Return: std::ops::Try = Result<Self, Self::Error>;

    fn try_from(value: T) -> Self::Return;
}
```

This would then allow for

```rust
impl TryFrom<i32> for Even {
    type Return = Eightball<Self, Odd>;

    fn try_from(num: i32) -> Self::Return {
        if num % 2 == 0 {
            Eightball::Yes(Even(num))
        } else {
            Eightball::No(Odd(num))
        }
    }
}
```

as well as the current

```rust
/// Shows this is **non-breaking change**: this is identical (text) to std impl
impl TryFrom<i8> for u8 {
    type Error = TryFromIntError;

    fn try_from(u: i8) -> Result<Self, Self::Error> {
        if u >= 0 {
            Ok(u as Self)
        } else {
            Err(TryFromIntError)
        }
    }
}
```

and even the following, which feels like it makes more sense than using a `PhraseTooShortError`

```rust
struct ThirdWord(String);

/// Could even return an Option
impl TryFrom<&str> for ThirdWord {
    type Return = Option<Self>;

    fn try_from(input: &str) -> Self::Return {
        input
            .split_whitespace()
            .nth(2)
            .map(|s| ThirdWord(s.to_string()))
    }
}
```

## Lint `clippy:TryMustUse`

`Result` (& `Poll`) are both annotated `#[must_use]` to ensure that residual cases are handled, or explicitly ignored. Arguably `Option` should also be `#[must_use]`, although it is not currently annotated as such.

As custom TryTypes begin to appear in 3rd-party crates, adding a (default warn) lint to recommend adding the `#[must_use]` annotation would save on downstream bug risks. It should be expected, that functions which return any TryType do so because of the risk of a residual, which should not be easily forgotten.

## `Poll<Result<!,E>>`
