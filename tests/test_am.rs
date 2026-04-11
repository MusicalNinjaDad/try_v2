#![allow(stable_features)]
#![feature(assert_matches)]

#[test]
fn what() {
    #[cfg(stable_assert_matches)]
    OnceCell::new(); //stable
    #[cfg(not(stable_assert_matches))]
    OnceCell::new(); //unstable
    #[cfg(assert_matches_in_root)]
    OnceCell::new(); //root
    #[cfg(assert_matches_in_module)]
    OnceCell::new(); //module
    panic!();
}

#[test]
#[cfg(assert_matches_in_root)]
fn in_root() {
    use std::assert_matches;
    assert_matches!(Some(4), Some(_));
}

#[test]
#[cfg(assert_matches_in_module)]
fn in_module() {
    use std::assert_matches::assert_matches;
    assert_matches!(Some(4), Some(_));
}
