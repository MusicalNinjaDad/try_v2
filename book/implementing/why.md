# Implementing Try

## Why?

The stdlib already provides `Option` and `Result` which generically cover most situations where you would need a *try-type*, so why bother implementing your own?

In the majority of cases you probably don't need to, and shouldn't. But ...

### Implementing other traits on Result

Type-alias-ing a `Result` is current idiom but this comes with one signifcant limitation. It is not possible to implement foreign traits on a type-alias. This is the case (wanting to `impl Termination` for the return type of `main`) that lead me to start down the rabbit-hole of "try" which lead to this book, crate.

### Flattening nested types

Rather than documenting:

```rust,ignore snippet
/// I _might_ have found somewhere that could contain duplicate info
///     Identifying duplicates _might_ have caused an error
///         And the answer _might_ be "no overlap"
type DuplicateData<T> = Option<Result<Option<T>, ValidErrors>>;

enum ValidErrors {
    Parsing(Box<dyn Error>),
    IO(Box<io::Error>),
}
```

It can be easier to reate, read, and reason about:

```rust,ignore snippet
enum DuplicateData<T> {
    Duplicate(T),
    NoCandidate,
    NoDuplicates,
    ParsingError(Box<dyn Error>),
    IOError(Box<io::Error>),
}
```
