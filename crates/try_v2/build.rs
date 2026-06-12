use ninja_build_rs::{Result, nightly::Nightly};

fn main() -> Result<()> {
    let ac = autocfg::new();

    ac.emit_unstable_feature("try_trait_v2");
    ac.emit_unstable_feature("try_trait_v2_residual");
    Ok(())
}
