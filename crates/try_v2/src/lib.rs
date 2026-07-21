#![cfg(all(has_try_trait_v2, has_try_trait_v2_residual))]

//! Provides derive macros for [Try](derive@Try), [FromResidual](derive@FromResidual) &
//! [IntoIterator](derive@IntoIterator).
//! See ([try_trait_v2](https://rust-lang.github.io/rfcs/3058-try-trait-v2.html)) for more details
//! of the underlying trait.
//!
//! ## Requires
//!
//!   - nightly
//!   - `#![feature(never_type)]`
//!   - `#![feature(try_trait_v2)]`
//!   - `#![feature(try_trait_v2_residual)]`
//!
//! ## Limitations on the annotated type
//!
//!   - must be an `enum`
//!   - must have _at least one_ generic type
//!   - the _first_ generic type must be the `Output` type (produced when not short-circuiting)
//!   - the output variant (does not short-circuit) must be the _first_ variant and store the output
//!     type as the _only unnamed_ field
//!   - no other variant can store the Output type (TODO #72 add a nice error message)
//!
//! See the individual documentation for [Try](derive@Try), [FromResidual](derive@FromResidual) &
//! [IntoIterator](derive@IntoIterator) for specifics
//! on the generated code.
//!
//! ## Example Usage
//!
//! ```rust, ignore TODO update
//! #![feature(never_type)]
//! #![feature(try_trait_v2)]
//! #![feature(try_trait_v2_residual)]
//! use try_v2::Try;
//!
//! #[derive(Try)]
//! #[FromResidual(Result<_, Self::Residual>)]
//! enum TestResult<T> {
//!     Ok(T),
//!     TestsFailed,
//!     OtherError(String)
//! }
//!
//! // Basic short-circuiting thanks to `#[derive(Try)]`
//! fn run_tests() -> TestResult<()> {
//!     TestResult::OtherError("oops!".to_string())?; // <- Function short-circuits here ...
//!     TestResult::TestsFailed?;
//!     TestResult::Ok(())
//! }
//!
//! assert!(matches!(run_tests(), TestResult::OtherError(msg) if msg == "oops!"));
//!
//!
//! // Conversion from std::result::Result thanks to `#[FromResidual(Result<_, Self::Residual>)]`
//! struct TestFailure {}
//!
//! impl<T> From<TestFailure> for TestResult<T> {
//!     fn from(err: TestFailure) -> Self {
//!         TestResult::TestsFailed
//!     }
//! }
//!
//! fn run_more_tests() -> TestResult<()> {
//!     std::result::Result::Err(TestFailure{})?; // <- Function short-circuits here & converts to a TestResult...
//!     TestResult::Ok(())
//! }
//!
//! assert!(matches!(run_more_tests(), TestResult::TestsFailed));
//! ```
//!
//! ## Stability & MSRV
//!
//! Given that this crate exposes an experimental API from std it makes use of experimental
//! features which require a nightly toolchain.
//!
//! In order to use this crate you must enable the features which it exposes:
//!
//! > 🔬 **Required Experimental Features**
//! >
//! >  - [`#![feature(never_type)]`](https://github.com/rust-lang/rust/issues/35121)
//! >  - [`#![feature(try_trait_v2)]`](https://github.com/rust-lang/rust/issues/84277)
//! >  - [`#![feature(try_trait_v2_residual)]`](https://github.com/rust-lang/rust/issues/91285)
//!
//! This crate makes use of the following experimental features in addition to those which it
//! directly supports:
//!
//! > 🔬 **Additional Experimental Features**
//! >
//! > - [`#![feature(if_let_guard)]`](https://github.com/rust-lang/rust/issues/51114) (stable since 1.95.0)
//! > - [`#![feature(let_chains)]`](https://github.com/rust-lang/rust/issues/139951) (stable since 1.88.0)
//! >
//! > This list includes any unstable features used by direct & transitive dependencies (currently, none).
//! >
//! > You do not need to enable these in your own code, the list is for information only.
//!
//! ### Stability guarantees
//!
//! We run automated tests **every month** to ensure no fundamental changes affect this crate and
//! test every PR against the current nightly, as well as the current equivalent beta & stable.
//! If you find an issue before we do, please
//! [raise an issue on github](https://github.com/MusicalNinjaDad/try_v2/issues).
//!
//! ### MSRV
//!
//! For those of you working with a pinned nightly (etc.) this crate supports every version of
//! edition 2024 (rust 1.85.1 onwards, released as stable on 2025-03-18). We use
//! [autocfg](https://crates.io/crates/autocfg/) to seamlessly handle features which have been
//! stabilised since then.
//!
//! ## Currently untested (may work, may not ...):
//!
//!   - `where` clauses
//!   - storing `Fn`s in variants

#[doc(inline)]
#[cfg(feature = "derive")]
pub use try_v2_derive::*;

#[doc(inline)]
#[cfg(feature = "traits")]
pub use try_v2_traits::*;
