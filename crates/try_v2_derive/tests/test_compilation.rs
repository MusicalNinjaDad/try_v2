use trybuild::TestCases;

#[test]
fn errors() {
    TestCases::new().compile_fail("tests/compilation/examples/fail_*.rs");
}
