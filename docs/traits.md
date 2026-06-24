# Overview

## Traits `Try`, `FromResidual`, `Residual`

The inter-operation of 3 traits is responsible for the correct function of a *try-type*

### [`Try`](https://doc.rust-lang.org/std/ops/trait.Try.html)

`trait Try` is responsible for:

- defining the `Output` (`Continue`) and `Residual` (`Break`) types
- splitting a *try-type* into `Continue` or `Break`
- generating an instance of the *try-type* by wrapping a value of the `Output` type

```rust,ignore snippet
pub trait Try: FromResidual {
    type Output;
    type Residual;

    // wraps a value of the `Output` type
    fn from_output(output: Self::Output) -> Self;

    // splits a *try-type* into `Continue` or `Break`
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output>;
}
```

### [`FromResidual`](https://doc.rust-lang.org/std/ops/trait.FromResidual.html)

`trait FromResidual<R>` is responsible for:

- generating and instance of the *try-type* from the relevant `Residual` type
- inter-conversion from one *try-type* `Residual` to a different *try-type*

```rust,ignore snippet
pub trait FromResidual<R = <Self as Try>::Residual> {
    // Generates a try-type from a residual
    fn from_residual(residual: R) -> Self;
}
```

### [`Residual`](https://doc.rust-lang.org/std/ops/trait.Residual.html)

`trait Residual<O>` is responsible for:

- defining the canonical *try-type* for a given `Residual`

```rust,ignore snippet
pub trait Residual<O>: Sized {
    type TryType: Try<Output = O, Residual = Self>;
}
```

## Overloading `?`

The compiler replaces `x?` with the following code (known as "desugaring")

```rust,ignore snippet
match Try::branch(x) {
    ControlFlow::Continue(v) => v,
    ControlFlow::Break(r) => return FromResidual::from_residual(r),
}
```

This means that *any* type which implements `Try` can be followed by a `?` in a function returning *any* type which has a suitable implementation of `FromResidual`
