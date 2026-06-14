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

Currently, using `try_fold()` on an `impl Iterator<Item=Poll<Result<T,E>>>` requires some interesting gymnastics

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
                Poll::Ready(n) => {
                    if let Poll::Ready(prev) = total {
                        Poll::Ready(prev + n)
                    } else {
                        unreachable!("we seed with Poll::Ready(0) and ignore Poll::Pending below")
                    }
                }
                // If we prefer to propogate Pending, then above can change to total.map()
                Poll::Pending => total,
            };
            Ok(total)
        },
    );
}
```

### Implications for homogeneity

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
