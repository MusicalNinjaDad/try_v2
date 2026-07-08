use ninja_build_rs::prelude::*;

fn main() -> Result<()> {
    let ac = autocfg::new();

    let allowed_features = cargo_allowed_features()?;
    ac.emit_unstable_feature(assert_matches, &allowed_features);
    ac.emit_unstable_feature(OtherFeature("let_chains".to_string()), &allowed_features); // stable 1.88.0 https://github.com/rust-lang/rust/issues/53667
    ac.emit_unstable_feature(iterator_try_collect, &allowed_features);
    ac.emit_unstable_feature(OtherFeature("if_let_guard".to_string()), &allowed_features); // stable 1.95.0 https://github.com/rust-lang/rust/issues/51114
    ac.emit_unstable_feature(never_type, &allowed_features);
    ac.emit_unstable_feature(try_trait_v2, &allowed_features);
    ac.emit_unstable_feature(try_trait_v2_residual, &allowed_features);
    Ok(())
}
