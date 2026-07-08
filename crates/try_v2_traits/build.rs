use ninja_build_rs::prelude::*;

fn main() -> Result<()> {
    let ac = autocfg::new();

    let allowed_features = cargo_allowed_features()?;
    ac.emit_unstable_feature(try_trait_v2, &allowed_features);
    ac.emit_unstable_feature(try_trait_v2_residual, &allowed_features);
    ac.emit_unstable_feature(OtherFeature("option_zip".to_string()), &allowed_features);
    Ok(())
}
