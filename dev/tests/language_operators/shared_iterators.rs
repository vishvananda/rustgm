//! Shared collection iteration uses exact selected contracts and checked originals.
use super::{mutation::frontend, native, rustc};
const CASES: &[(&str, bool)] = &[
    ("shared-array", true),
    ("generic-shared-array", true),
    ("shared-slice", true),
    ("shared-slice-no-copy-bound", true),
    ("slice-iter", true),
    ("shared-write-reject", false),
    ("named-slice-iter", true),
    ("slice-rest-length", true),
    ("slice-qualified", true),
    ("array-qualified-reject", false),
    ("array-trait-wins", true),
    ("iter-private-constructor", false),
    ("iter-private-rest", true),
    ("iter-not-copy", false),
    ("iter-clone-no-element-clone", true),
    ("item-outlives-iterator", true),
    ("items-survive-next", true),
    ("generic-source-lifetime", true),
    ("shared-item-conflicts-mutation", false),
    ("iterator-source-escape", false),
    ("iterator-private-field", false),
    ("slice-iter-reified", true),
    ("shared-empty-zst", true),
    ("iter-method-generic-reject", false),
    ("formation-evaluated-once", true),
    ("shared-noncopy-drop", true),
];
fn source(name: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(format!(
        "{}/../pa26/tests/spec/iteration/shared-collections/{name}.rs",
        env!("CARGO_MANIFEST_DIR")
    ))
}
#[test]
fn shared_collection_outcomes_keep_items_borrows_and_lookup()
-> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, accepted) in CASES {
        let text = source(name)?;
        rustc::qualify(
            &format!("parallel/pa26/shared-iteration/source/{name}"),
            &text,
            accepted,
        )?;
        let result = frontend::checked(&text);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn shared_collections_execute_once_with_noncopy_items_and_source_lifetimes()
-> Result<(), Box<dyn std::error::Error>> {
    let sources = CASES
        .iter()
        .filter(|(_, accepted)| *accepted)
        .map(|(name, _)| Ok((*name, source(name)?)))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    let cases = sources
        .iter()
        .map(|(name, text)| (*name, text.as_str(), false))
        .collect::<Vec<_>>();
    native::check_literal_cases("parallel/pa26/shared-iteration/native", &cases)
}
