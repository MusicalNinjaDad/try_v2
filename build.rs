use std::fmt::Display;

use autocfg::AutoCfg;

extern crate autocfg;

fn main() {
    let ac = autocfg::new();

    ac.emit_unstable_feature("assert_matches");
    AssertMatchesLocation::emit_possibilities();
    if let Some(location) = ac.assert_matches_location() {
        autocfg::emit(&location.to_string())
    }

    ac.emit_unstable_feature("let_chains");

    ac.emit_unstable_feature("if_let_guard");
}

enum AssertMatchesLocation {
    /// Macro is at `std::assert_matches`
    Root,
    /// Macro is at `std::assert_matches::assert_matches`
    Module,
}

impl Display for AssertMatchesLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssertMatchesLocation::Root => write!(f, "assert_matches_in_root"),
            AssertMatchesLocation::Module => write!(f, "assert_matches_in_module"),
        }
    }
}

impl AssertMatchesLocation {
    fn emit_possibilities() {
        autocfg::emit_possibility("assert_matches_in_root");
        autocfg::emit_possibility("assert_matches_in_module");
    }
}

trait Nightly {
    /// Identify whether a an experimental feature flag is available _and_ required on nightly.
    /// Always fails if feature flags are unavailable.
    ///
    /// ## Usage:
    /// To be used at top-level crate via `#![cfg_attr(unstable_foo, feature(foo))]`
    fn emit_unstable_feature(&self, feature: &'static str);

    /// Location of assert_matches!() macro. Stabilisation was reverted at last minute
    /// on 2026-04-10, leaving the macro in the new planned location.
    fn assert_matches_location(&self) -> Option<AssertMatchesLocation>;
}

impl Nightly for AutoCfg {
    fn emit_unstable_feature(&self, feature: &'static str) {
        let cfg = format!("unstable_{feature}");
        let code = format!(
            r#"
        #![deny(stable_features)]
        #![feature({feature})]
        "#
        );
        autocfg::emit_possibility(&cfg);
        if self.probe_raw(&code).is_ok() {
            autocfg::emit(&cfg);
        }
    }

    fn assert_matches_location(&self) -> Option<AssertMatchesLocation> {
        let in_root = r#"
        #![allow(stable_features)]
        #![feature(assert_matches)]
        use std::assert_matches;

        fn main() {
            assert_matches!(Some(4), Some(_));
        }
            "#;

        let in_module = r#"
        #![allow(stable_features)]
        #![feature(assert_matches)]
        use std::assert_matches::assert_matches;

        fn main() {
            assert_matches!(Some(4), Some(_));
        }
            "#;

        if self.probe_raw(in_root).is_ok() {
            Some(AssertMatchesLocation::Root)
        } else if self.probe_raw(in_module).is_ok() {
            Some(AssertMatchesLocation::Module)
        } else {
            None
        }
    }
}
