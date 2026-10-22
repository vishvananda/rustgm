//! Array/slice equality uses selected element evidence and ordinary checked loops.
use super::{mutation::frontend, native, rustc};
const CASES: &[(&str, bool)] = &[
    ("array-equality", true),
    ("slice-equality", true),
    ("array-slice", true),
    ("slice-array", true),
    ("heterogeneous-array", true),
    ("heterogeneous-slice", true),
    ("heterogeneous-array-slice", true),
    ("different-array-length", false),
    ("different-primitives", false),
    ("empty-missing-element-bound", false),
    ("float-equality", true),
    ("float-eq-bound", false),
    ("array-ne-override", true),
    ("slice-ne-override", true),
    ("generic-array", true),
    ("generic-slice", true),
    ("array-qualified-reified", true),
    ("slice-qualified-reified", true),
    ("nested-equality", true),
    ("aliased-shapes", true),
    ("zero-sized-elements", true),
    ("rest-pattern-metadata", true),
    ("local-source-implementation", true),
    ("shared-slice-reference", true),
    ("mutable-slice-reference", true),
    ("array-reference-rejected", false),
    ("reference-array-rejected", false),
    ("heterogeneous-array-reference", true),
    ("heterogeneous-reference-array", true),
    ("lookalike-element-trait", false),
    ("empty-unpaid-eq", false),
    ("paid-lifetime", true),
    ("unpaid-lifetime", false),
    ("inferred-array-length", true),
    ("distinct-symbolic-lengths", false),
    ("explicit-array-bound", true),
    ("latent-module-origin", true),
    ("latent-unrelated-query", true),
    ("latent-macro-origin", true),
    ("eager-canonical-impl", true),
    ("eq-prerequisite", true),
    ("ord-prerequisite", true),
    ("alias-dot-wrong-rhs", false),
];
fn source(name: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(format!(
        "{}/../pa26/tests/spec/sequence-equality/{name}.rs",
        env!("CARGO_MANIFEST_DIR")
    ))
}
#[test]
fn sequence_equality_matches_declared_element_contracts() -> Result<(), Box<dyn std::error::Error>>
{
    let mut failures = Vec::new();
    for &(name, accepted) in CASES {
        let source = source(name)?;
        rustc::qualify(&format!("sequence-equality-{name}"), &source, accepted)?;
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
fn sequence_equality_preserves_element_dispatch_and_metadata()
-> Result<(), Box<dyn std::error::Error>> {
    let sources = CASES
        .iter()
        .filter(|case| case.1)
        .map(|&(name, _)| Ok((name, source(name)?)))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    let cases = sources
        .iter()
        .map(|(name, source)| {
            Ok((
                *name,
                source
                    .strip_prefix(native::CHECK)
                    .ok_or("missing native check prefix")?,
                false,
            ))
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    native::check_cases("parallel/pa26-sequence-equality/native", &cases)
}
