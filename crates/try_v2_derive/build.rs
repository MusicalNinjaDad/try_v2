use ninja_build_rs::{Result, nightly::Nightly};

fn main() -> Result<()> {
    let ac = autocfg::new();

    ac.emit_unstable_feature("assert_matches");
    ac.emit_unstable_feature("let_chains"); // stable 1.88.0 https://github.com/rust-lang/rust/issues/53667
    ac.emit_unstable_feature("iterator_try_collect");
    ac.emit_unstable_feature("if_let_guard"); // stable 1.95.0 https://github.com/rust-lang/rust/issues/51114
    ac.emit_unstable_feature("never_type");
    ac.emit_unstable_feature("try_trait_v2");
    ac.emit_unstable_feature("try_trait_v2_residual");
    Ok(())
}
