//! Canonical reference comparison blankets retain same-member pointee dispatch.
use super::{mutation::frontend, native, rustc};
use std::path::{Path, PathBuf};
const CASES: &[(&str, bool, bool)] = &[
    ("contract-partialeq-shared-shared", true, false),
    ("contract-partialeq-shared-mutable", true, false),
    ("contract-partialeq-mutable-shared", true, false),
    ("contract-partialeq-mutable-mutable", true, false),
    ("contract-partialord-shared-shared", true, false),
    ("contract-partialord-shared-mutable", false, false),
    ("contract-partialord-mutable-shared", false, false),
    ("contract-partialord-mutable-mutable", true, false),
    ("contract-eq-shared", true, false),
    ("contract-eq-mutable", true, false),
    ("contract-ord-shared", true, false),
    ("contract-ord-mutable", true, false),
    ("contract-generic-reference-gap", true, false),
    ("contract-unsized-pointee", true, false),
    ("contract-nested-shared", true, false),
    ("contract-nested-mutable", true, false),
    ("contract-nested-mixed-same-shape", true, false),
    ("contract-nested-cross-shape", true, false),
    ("contract-not-eq-pointee", false, false),
    ("contract-float-eq", false, false),
    ("contract-float-ord", false, false),
    ("eq-shared-shared", true, true),
    ("eq-shared-mutable", true, true),
    ("eq-mutable-shared", true, true),
    ("eq-mutable-mutable", true, true),
    ("generic-reference-gap", true, true),
    ("mutable-usable-after", true, true),
    ("heterogeneous-overrides", true, true),
    ("generic-heterogeneous", true, true),
    ("nested-shared", true, true),
    ("nested-mutable", true, true),
    ("eq-cross-nested", true, true),
    ("user-reference-impl", true, true),
    ("reference-impl-overlap", false, false),
    ("intermediate-user-impl", true, true),
    ("pointee-lifetime", true, true),
    ("unpaid-lifetime", false, false),
    ("wrong-reified-outer", false, false),
    ("same-named-trait", true, true),
    ("same-named-no-operator", false, false),
    ("coherence-reified-fixed-inner", true, true),
    ("coherence-source-before-pointee", false, false),
    ("coherence-pointee-before-source", false, false),
    ("coherence-generic-possible-overlap", false, false),
    ("coherence-generic-disjoint-local", true, false),
    ("coherence-conditional-foreign-bound", false, false),
    ("coherence-unsized-opposing", true, false),
    ("coherence-sized-opposing", false, false),
    ("coherence-generic-unresolved-use", true, true),
    ("borrowed-helpers", true, true),
    ("reference-ties", true, true),
    ("owned-helper-overrides-not-forwarded", true, true),
    ("noncopy-mutable-pointee", true, true),
    ("aliased-blanket", true, true),
    ("shared-loan-protected", false, false),
    ("unrelated-recursive-disjoint", true, false),
    ("dot-reference-member", true, true),
    ("const-call-not-eligible", false, false),
    ("nan-forwarding", true, true),
    ("same-named-trait-owned", true, true),
    ("dot-generic-reference", true, true),
];
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../pa26/tests/spec/reference-comparison/{name}.rs"))
}
#[test]
fn reference_comparison_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, accepted, _) in CASES {
        let source = std::fs::read_to_string(fixture(name))?;
        rustc::qualify(
            &format!("parallel/pa26/reference-comparison/independent/{name}"),
            &source,
            accepted,
        )?;
        let checked = frontend::checked(&source);
        let implemented = accepted && name != "contract-unsized-pointee";
        if checked.is_ok() != implemented {
            failures.push(format!("{name}: {:?}", checked.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn reference_comparison_executes() -> Result<(), Box<dyn std::error::Error>> {
    let names = CASES
        .iter()
        .filter(|case| case.1 && case.2)
        .collect::<Vec<_>>();
    let sources = names
        .iter()
        .map(|case| std::fs::read_to_string(fixture(case.0)))
        .collect::<Result<Vec<_>, _>>()?;
    let cases = names
        .iter()
        .zip(&sources)
        .map(|(case, source)| {
            Ok((
                case.0,
                source.strip_prefix(native::CHECK).ok_or("check prefix")?,
                false,
            ))
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    native::check_cases("parallel/pa26/reference-comparison/native", &cases)
}
