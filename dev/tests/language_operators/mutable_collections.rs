//! Mutable slice/array iteration keeps selected contracts and disjoint yielded loans.
use super::{mutation::frontend, native, rustc};
const CASES: &[(&str, bool)] = &[
    ("mutable-array", true),
    ("mutable-slice", true),
    ("slice-iter-mut", true),
    ("slice-reborrow", true),
    ("slice-moved-reference-reject", false),
    ("slice-double-mutable-reject", false),
    ("successive-live-items", true),
    ("item-outlives-iterator", true),
    ("source-write-reject", false),
    ("named-iter-mut", true),
    ("qualified-next", true),
    ("reified-next", true),
    ("empty-zst", true),
    ("noncopy-drop", true),
    ("source-lifetime-shortening", true),
    ("source-lifetime-widening-reject", false),
    ("pointee-invariance-reject", false),
    ("iter-mut-not-copy", false),
    ("iter-mut-not-clone", false),
    ("source-inherent-winner", true),
    ("array-trait-winner", true),
    ("source-lookalike", true),
    ("no-prelude-import", true),
    ("no-prelude-missing", false),
    ("generic-array-extent", true),
    ("slice-rest-metadata", true),
    ("formation-evaluated-once", true),
];
fn source(name: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(format!(
        "{}/../pa26/tests/spec/iteration/mutable-collections/{name}.rs",
        env!("CARGO_MANIFEST_DIR")
    ))
}
#[test]
fn mutable_collection_outcomes_keep_unique_items_and_lookup()
-> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, accepted) in CASES {
        let text = source(name)?;
        rustc::qualify(
            &format!("parallel/pa26/mutable-iteration/source/{name}"),
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
fn mutable_collections_execute_with_live_items_and_noncopy_elements()
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
    native::check_literal_cases("parallel/pa26/mutable-iteration/native", &cases)
}
