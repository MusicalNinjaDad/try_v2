# Changelog try_v2

## [v0.7.5]

### Bugfix

- Remove tempory version pin on `proc_macro2_diagnostic`, which was required to publish v0.7.4 without circular dependency

## [v0.7.4]

### Bugfix

- Made `derive` a (default) feature, to remove circular dependency risk with `proc_macro2_diagnostic`

## [v0.7.3]

### Bugfix

- Cfg-gate entire crate to `has_try_trait_v2` (& friends) - allows to be a dependency for crates which also need to compile on stable

## [v0.7.2]

### Bugfix

- Updated to latest `proc_macro2_diagnostic`

## [v0.7.1]

### New features

- Further improved ergonomics for overloading overlapping trait methods (e.g. `unwrap` now re-uses `extract`)

## [v0.7.0]

### Bugfixes

- Improved ergonomics for overloading provided impls from traits

### Breaking changes

- Moved `<T>` to trait signature for `Extract<T>` and `Transform<T>`

## [v0.6.0]

### New features

- Extend `trait Transform`
- Add `trait Extract`

### Breaking changes

- Move `output` from `Transform` to `Extract`

## [v0.5.1]

### Bugfixes

- Simplify `transpose()` signature (`where` clause)

## [v0.5.0]

### New features

- Add `trait Transform` (Forces move of derive macros to own crate & re-export)

### Breaking changes

- Removed derive `Try_Methods` in favour of trait-based implementation

## [v0.4.2]

### Bugfixes

- Fix missing files in package

## [v0.4.1]

### Documentation & Testing

- Document & continuously validate stability guarantees & experimental feature usage

### Bugfixes

- Remove lint warnings for stable features on stricter nightly toolchains

## [v0.4.0]

### Breaking changes

- `Try_ConvertResult` now requires `E: Into<MyTry<!>>` (required for bug fix below and to reduce risk of accidentally converting an Error into an OK)

### New features

- Add `Try_Iterator` to derive `IntoIterator` and `FromIterator`

### Bugfixes

- Allow for functions returning `Result<T, MyTry<!>>` to be `?`-ed in functions returning `MyTry<U>`

## [v0.3.5]

- Update `proc_macro2_diagnostic` to 0.2.0
- Emit warning if enum not marked `#[must_use]`
