extern crate autocfg;

fn main() {
    let ac = autocfg::new();
    ac.emit_path_cfg("std::assert_matches", "stable_assert_matches");

    autocfg::emit_possibility("stable_let_chains");
    let stable_let_chains = r#"
    #![deny(stable_features)]
    #![feature(let_chains)]
    "#;
    if ac.probe_raw(stable_let_chains).is_err() {
        autocfg::emit("stable_let_chains");
    };
}
