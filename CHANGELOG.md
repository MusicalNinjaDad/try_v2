# Changelog try_v2

## [0.4.0]

### Breaking changes

- `Try_ConvertResult` now requires `E: Into<MyTry<!>>` (required for bug fix below and to reduce risk of accidentally converting an Error into an OK)

### Bugfixes

- Allow for functions returning `Result<T, MyTry<!>>` to be `?`-ed in functions returning `MyTry<U>`

## [v0.3.5]

- Update `proc_macro2_diagnostic` to 0.2.0
- Emit warning if enum not marked `#[must_use]`
