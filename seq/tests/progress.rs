#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-parse-header.rs");
    t.pass("tests/02-parse-body.rs");
    t.compile_fail("tests/03-expand-four-errors.rs");
    t.pass("tests/04-paste-ident.rs");
    t.pass("tests/04-paste-ident2.rs");
    t.pass("tests/05-repeat-section.rs");
    t.pass("tests/05-repeat-section2.rs");
    t.pass("tests/06-init-array.rs");
    t.pass("tests/07-inclusive-range.rs");
    t.compile_fail("tests/08-ident-span.rs");
    t.pass("tests/09-interaction-with-macrorules.rs");
    t.compile_fail("tests/09-interaction-with-macrorules2.rs");
    t.compile_fail("tests/10-literal-range-parse-err.rs");
}
