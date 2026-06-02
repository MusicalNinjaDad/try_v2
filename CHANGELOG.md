# Changelog try_v2

## [v0.5.0]

### New features

- Add `trait Transform` (Forces move of derive macros to own crate & re-export)

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
