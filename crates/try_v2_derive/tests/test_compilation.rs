use trybuild::TestCases;

#[test]
fn errors() {
    TestCases::new().compile_fail("tests/compilation/fail_*.rs");
}
