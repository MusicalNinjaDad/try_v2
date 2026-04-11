extern crate autocfg;

fn main() {
    let ac = autocfg::new();
    ac.emit_path_cfg("std::assert_matches", "stable_assert_matches");
}
