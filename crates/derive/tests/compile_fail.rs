//! Derive inputs the macro refuses, each with the message it gives

#[test]
fn refused_inputs() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/*.rs");
}
