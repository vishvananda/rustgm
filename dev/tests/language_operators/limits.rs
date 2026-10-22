//! Primitive constants retain canonical types without source initializers.
use super::{mutation::frontend, native, rustc};
use std::path::{Path, PathBuf};
const CASES: &[(&str, bool, bool)] = &[
    ("max-u8", true, true),
    ("max-u16", true, true),
    ("max-u32", true, true),
    ("max-u64", true, true),
    ("max-u128", true, true),
    ("max-usize", true, true),
    ("max-i8", true, true),
    ("max-i16", true, true),
    ("max-i32", true, true),
    ("max-i64", true, true),
    ("max-i128", true, true),
    ("max-isize", true, true),
    ("type-alias", true, true),
    ("nested-alias", true, true),
    ("inherent-self", true, true),
    ("qualified-type", true, true),
    ("core-primitive", true, true),
    ("std-primitive", true, true),
    ("no-prelude", true, true),
    ("lexical-lookalike", true, true),
    ("nominal-alias-lookalike", true, true),
    ("explicit-trait-owner", true, true),
    ("trait-generic-owner", true, true),
    ("promotion", true, true),
    ("mutable-temporary", true, true),
    ("constant-pattern", true, true),
    ("const-generic", true, true),
    ("inline-constant", true, true),
    ("wrong-result", false, false),
    ("member-arguments", false, false),
    ("primitive-arguments", false, false),
    ("not-callable", false, false),
    ("assign-constant", false, false),
    ("bool-no-member", false, false),
    ("str-no-member", false, false),
    ("unbounded-generic", false, false),
    ("primitive-import", false, false),
    ("float-required", false, true),
    ("char-required", false, true),
    ("impl-self", true, true),
    ("impl-self-constant-precedence", true, true),
    ("impl-self-explicit-trait", true, true),
    ("trait-default-self", true, true),
    ("forward-alias", true, true),
    ("shadow-with-canonical-escape", true, true),
    ("source-core-lookalike", true, true),
    ("direct-pattern", true, true),
    ("range-pattern", true, true),
    ("self-in-initializer", true, true),
    ("aliased-self-priority", true, true),
    ("empty-prefix-arguments", true, true),
    ("empty-member-arguments", true, true),
    ("no-implicit-prelude-alias", true, true),
    ("bool-trait-constant", true, true),
    ("float-trait-priority", false, false),
    ("alias-nominal-constant", true, true),
    ("generic-alias", true, true),
    ("nested-generic-alias", true, true),
    ("projected-concrete-alias", true, true),
    ("generic-alias-wrong-arity", false, false),
    ("generic-alias-unresolved", false, false),
    ("generic-nominal-alias", true, true),
];
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../pa26/tests/spec/primitive-constants/{name}.rs"))
}
#[test]
fn primitive_constants_match_declared_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, accepted, independent) in CASES {
        let source = std::fs::read_to_string(fixture(name))?;
        rustc::qualify(
            &format!("parallel/pa26/primitive-constants/source/{name}"),
            &source,
            independent,
        )?;
        let result = frontend::checked(&source);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn primitive_constants_execute_with_exact_lookup_and_widths()
-> Result<(), Box<dyn std::error::Error>> {
    let check = "fn check(b:bool){if !b{let z=0;let _=1/z;}}";
    let sources = CASES
        .iter()
        .filter(|case| case.1 && !matches!(case.0, "no-prelude" | "no-implicit-prelude-alias"))
        .map(|case| Ok((case.0, std::fs::read_to_string(fixture(case.0))?)))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    let cases = sources
        .iter()
        .map(|(name, source)| (*name, source.strip_prefix(check).unwrap_or(source), false))
        .collect::<Vec<_>>();
    native::check_cases("parallel/pa26/primitive-constants/native", &cases)
}
