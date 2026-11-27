//! Derive inputs the macro refuses, each with the message it gives

#[test]
#[cfg_attr(miri, ignore = "trybuild runs cargo, which Miri isolation refuses")]
fn refused_inputs() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/*.rs");
}
