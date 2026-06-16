# Overview

## Traits `Try`, `FromResidual`, `Residual`

The inter-operation of 3 traits is responsible for the correct function of a *try-type*

### [`Try`](https://doc.rust-lang.org/std/ops/trait.Try.html)

`trait Try` is responsible for:

- defining the `Output` (`Continue`) and `Residual` (`Break`) types
- splitting a *try-type* into `Continue` or `Break`
- generating an instance of the *try-type* from the `Output` type

### [`FromResidual`](https://doc.rust-lang.org/std/ops/trait.FromResidual.html)

`trait FromResidual<R>` is responsible for:

- generating and instance of the *try-type* from the relevant `Residual` type
- inter-conversion from one *try-type* `Residual` to a different *try-type*

### [`Residual`](https://doc.rust-lang.org/std/ops/trait.Residual.html)

`trait Residual<O>` is responsible for:

- defining the canonical *try-type* for a given `Residual`

## Overloading `?`

The compiler replaces `x?` with the following code (known as "desugaring")

```rust
match Try::branch(x) {
    ControlFlow::Continue(v) => v,
    ControlFlow::Break(r) => return FromResidual::from_residual(r),
}
```

This means that *any* type which implements `Try` can be followed by a `?` in a function returning *any* type which has a suitable implementation of `FromResidual`
