use autocfg::AutoCfg;

extern crate autocfg;

fn main() {
    let ac = autocfg::new();
    ac.emit_unstable_feature("assert_matches");
    stable_feature(&ac, "assert_matches");
    assert_matches_in_module(&ac);
    assert_matches_in_root(&ac);

    ac.emit_unstable_feature("let_chains");
    stable_feature(&ac, "let_chains");

    ac.emit_unstable_feature("if_let_guard");
    stable_feature(&ac, "if_let_guard");
}

trait UnstableFeature {
    /// Identify whether a an experimental feature flag is available _and_ required on nightly.
    /// Always fails if feature flags are unavailable.
    ///
    /// ## Usage:
    /// To be used at top-level crate via `#![cfg_attr(unstable_foo, feature(foo))]`
    fn emit_unstable_feature(&self, feature: &'static str);
}

impl UnstableFeature for AutoCfg {
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
}

fn stable_feature(ac: &AutoCfg, feature: &'static str) {
    let cfg = format!("stable_{feature}");
    let deny = format!(
        r#"
    #![deny(stable_features)]
    #![feature({feature})]
    "#
    );

    let allow = format!(
        r#"
    #![allow(stable_features)]
    #![feature({feature})]
    "#
    );

    autocfg::emit_possibility(&cfg);
    if ac.probe_raw(&deny).is_err() && ac.probe_raw(&allow).is_ok() {
        autocfg::emit(&cfg);
    }
}

fn assert_matches_in_root(ac: &AutoCfg) {
    let cfg = "assert_matches_in_root";
    let code = r#"
    #![allow(stable_features)]
    #![feature(assert_matches)]
    use std::assert_matches;

    fn main() {
        assert_matches!(Some(4), Some(_));
    }
        "#;
    autocfg::emit_possibility(cfg);
    if ac.probe_raw(code).is_ok() {
        autocfg::emit(cfg);
    }
}

fn assert_matches_in_module(ac: &AutoCfg) {
    let cfg = "assert_matches_in_module";
    let code = r#"
    #![allow(stable_features)]
    #![feature(assert_matches)]
    use std::assert_matches::assert_matches;

    fn main() {
        assert_matches!(Some(4), Some(_));
    }
        "#;
    autocfg::emit_possibility(cfg);
    if ac.probe_raw(code).is_ok() {
        autocfg::emit(cfg);
    }
}
