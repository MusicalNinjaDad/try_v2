# Changelog try_v2

## [v0.9.0]

### Breaking changes / New Features

- derive `FromResidual` replaces `Try_ConvertResult`, works on any TryType
- derive `IntoIterator` replaces `Try_Iterator`, works on any TryType
- `FromIterator` removed
- methods generates `iter`, `iter_mut`, `as_ref`, `as_mut`, `as_deref`, `as_deref_mut`
- remove `ConvertResult`
- remove deriving `FromResidual` to convert *to* a `Result::Err`

## Bugfix

- Update to ninja-build_rs v0.2.0 to (partially) respect cargo unstable.allow-features

## Open tasks

- Finalise documentation with all updates
- Bump `proc_macro2_diagnostic` once it relies on this version to fully respect cargo unstable.allow-features

## [v0.8.2]

### New features

- Added `impl Transform` & `impl Extract` for `Option`, `Result` & `ControlFlow`

### Technical changes

- Improved documentation regarding `map_residual`

## [v0.8.1]

### New features

- Added `map_residual` to `trait Transform`

### Bugfix

- Ensure downstream crates can depend on both `try_v2` and `proc_macro2_diagnostic`

## [v0.8.0]

### Breaking changes

- Moved traits to feature-gated re-export: feature = `traits` (default); crate: `try_v2_traits`

## [v0.7.5]

### Bugfix

- Remove temporary version pin on `proc_macro2_diagnostic`, which was required to publish v0.7.4 without circular dependency

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
