use trybuild::TestCases;

#[test]
fn errors() {
    TestCases::new().compile_fail("tests/compilation/fail_*.rs");
}

#[test]
fn valid() {
    TestCases::new().pass("tests/compilation/pass_*.rs");
}