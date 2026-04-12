use std::fs;

use dircpy::copy_dir;
use tempfile::tempdir;

#[test]
fn fmt_fixture() {
    let tmp = tempdir().expect("couldn't create temp dir for test");
    copy_dir("tests/fixture", tmp.path()).expect("couldn't copy fixture");
    let original = fs::read_to_string("tests/fixture/src/lib.rs").unwrap();
    let copied = fs::read_to_string(tmp.path().join("src/lib.rs")).unwrap();
    assert_eq!(original,copied);
    fmt(tmp.path()).unwrap();
    assert_ne!(original,copied);
}
