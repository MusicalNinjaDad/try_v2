use autocfg::AutoCfg;

extern crate autocfg;

fn main() {
    let ac = autocfg::new();
    stable_feature(&ac, "assert_matches");

    autocfg::emit_possibility("assert_matches_in_module");
    let code = r#"
        #![allow(stable_features)]
        #![feature(assert_matches)]
        use std::assert_matches::assert_matches;
    "#;
    if ac.probe_raw(code).is_ok() {
        autocfg::emit("assert_matches_in_module");
    }

    stable_feature(&ac, "let_chains");

    stable_feature(&ac, "if_let_guard");
}

fn stable_feature(ac: &AutoCfg, feature: &'static str) {
    let cfg = format!("stable_{feature}");
    let code = format!(
        r#"
    #![deny(stable_features)]
    #![feature({feature})]
    "#
    );

    autocfg::emit_possibility(&cfg);
    if ac.probe_raw(&code).is_err() {
        autocfg::emit(&cfg);
    }
}
