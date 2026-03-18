use trybuild::TestCases;

#[test]
fn errors() {
    TestCases::new().compile_fail("tests/errors/*.rs");
}
