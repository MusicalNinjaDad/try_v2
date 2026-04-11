use trybuild::TestCases;

#[test]
fn errors() {
    TestCases::new().compile_fail("tests/compilation/examples/fail_*.rs");
}

#[test]
fn valid() {
    TestCases::new().pass("tests/compilation/examples/pass_*.rs");
}
