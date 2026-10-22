//! Executable iterator cloning retains canonical selection and ordinary shared-view proof.
use super::{mutation::frontend, native, rustc};
const CASES: &[(&str, bool)] = &[
    ("original-nonclone-elements", true),
    ("partial-state", true),
    ("generic-clone-bound", true),
    ("qualified-clone", true),
    ("qualified-iter-clone", true),
    ("reified-clone", true),
    ("item-survives-clone-owner", true),
    ("empty-zst", true),
    ("noncopy-drop", true),
    ("no-prelude-import", true),
    ("no-prelude-missing", false),
    ("source-shadow", false),
    ("source-lookalike", true),
    ("competing-trait", false),
    ("source-mutation", false),
    ("source-escape", false),
    ("iter-still-not-copy", false),
    ("plain-nonclone", false),
    ("after-exhaustion", true),
    ("live-borrowed-item", true),
    ("qualified-source-clone", true),
    ("qualified-generic-bound", true),
    ("clone-supertrait-sized", true),
    ("abstract-generic", true),
    ("no-prelude-generic", true),
    ("source-winner-with-iter", true),
    ("inherent-winner-with-iter", true),
    ("clone-const-reject", false),
    ("clone-foreign-pointer-reject", false),
    ("macro-canonical-origin", true),
    ("std-alias-qualified", true),
];
fn source(name: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(format!(
        "{}/../pa26/tests/spec/iteration/clone/{name}.rs",
        env!("CARGO_MANIFEST_DIR")
    ))
}
#[test]
fn iterator_clone_matches_declared_lookup_lifetime_and_permission_outcomes()
-> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, accepted) in CASES {
        let text = source(name)?;
        rustc::qualify(
            &format!("parallel/pa26/iterator-clone/source/{name}"),
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
fn iterator_clone_executes_independent_cursors_and_keeps_borrowed_items()
-> Result<(), Box<dyn std::error::Error>> {
    let sources = CASES
        .iter()
        .filter(|(name, accepted)| *accepted && *name != "abstract-generic")
        .map(|(name, _)| Ok((*name, source(name)?)))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    let cases = sources
        .iter()
        .map(|(name, text)| (*name, text.as_str(), false))
        .collect::<Vec<_>>();
    native::check_literal_cases("parallel/pa26/iterator-clone/native", &cases)
}
#[test]
fn marker_only_concrete_targets_remain_required_executable_gaps()
-> Result<(), Box<dyn std::error::Error>> {
    for (name, text) in [
        (
            "derived",
            "#[derive(Clone)]struct V;fn duplicate<T:Clone>(x:&T)->T{<T as Clone>::clone(x)}fn main(){let _=duplicate(&V);}",
        ),
        (
            "reference",
            "struct V;fn duplicate<T:Clone>(x:&T)->T{<T as Clone>::clone(x)}fn main(){let v=V;let _=duplicate(&&v);}",
        ),
    ] {
        rustc::qualify(
            &format!("parallel/pa26/iterator-clone/required/{name}"),
            text,
            true,
        )?;
        let error = frontend::checked(text)
            .err()
            .ok_or("unproved concrete marker dispatch was accepted")?;
        assert!(
            error.contains("concrete dispatch cannot use a symbolic assumption"),
            "{name}: {error}"
        );
    }
    Ok(())
}
