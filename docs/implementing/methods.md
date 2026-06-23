# Implementing Try

## Making TryTypes usable

While having a type which responds to `?` is nice, sooner or later you will start to realise that stdlib have put a lot of very useful methods on `Option` & `Result`. Your TryType doesn't have these and so code very quickly starts to hard to write, or tightly coupled & verbose.

[`try_v2`](https://crates.io/crates/try_v2) offers two traits with provided implementations of a range of method for any TryType, both inspired by the methods provided on `Option` & `Result`. The names should be recognisable.

### trait Extract

Methods for extracting the wrapped value

```rust
pub trait Extract<T>: Sized + Try<Output = T> {
    // Provided methods
    fn output(self) -> Option<T> { ... }
    fn unwrap(self) -> T { ... }
    fn expect(self, msg: &str) -> T { ... }
    fn unwrap_or(self, default: T) -> T { ... }
    fn unwrap_or_default(self) -> T { ... }
    fn unwrap_or_else<F>(self, f: F) -> T { ... }
}
```

### trait Transform

Methods for transforming TryTypes.

```rust
pub trait Transform<T>: Sized + Try<Output = T> {
    // Provided methods
    fn flatten(self) -> T { ... }
    fn inspect<F>(self, f: F) -> Self { ... }
    fn map<X, U, F>(self, f: F) -> X { ... }
    fn map_or<U, F>(self, default: U, f: F) -> U { ... }
    fn map_or_else<U, D, F>(self, default: D, f: F) -> U { ... }
    fn map_residual<F, X, G>(self, f: F) -> X { ... }
    fn transpose<X>(self) -> X { ... }
    fn zip<X, Y>(self, other: Y) -> X { ... }
    fn zip_with<X, Y, F, G>(self, other: Y, f: F) -> X { ... }
    fn and<Y>(self, other: Y) -> Y { ... }
    fn and_then<Y, F>(self, f: F) -> Y { ... }
    fn or<Y>(self, other: Y) -> Y { ... }
    fn or_else<Y, F>(self, f: F) -> Y { ... }
}
```

Generic type conventions used in signatures (in standard order):

- `X` the *canonical TryType* returned
- `Y` the other TryType
- `T` the `Output` type for `Self`
- `U` the other `Output` type
- `F` a function/closure passed as a parameter
- `G` the return type of `F`
- `R` *never used* to avoid confusion with "Return" vs. "Residual"
