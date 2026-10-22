//! Canonical iteration methods use the existing lexical and prelude candidate index.
use super::{mutation::frontend, native, rustc};
use std::path::Path;
const CASES: &[(&str, bool)] = &[
    ("prelude-next", true),
    ("explicit-next", true),
    ("aliased-next", true),
    ("anonymous-next", true),
    ("no-prelude-next", true),
    ("no-prelude-missing-reject", false),
    ("source-lookalike", true),
    ("user-canonical-ambiguity-reject", false),
    ("shadowed-prelude-ambiguity-reject", false),
    ("inherent-wins", true),
    ("generic-bound", true),
    ("shared-receiver-reject", false),
    ("prelude-into-iter", true),
    ("generic-reference-item", true),
];
fn source(name: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa26/tests/spec/iteration-visibility/{name}.rs")),
    )
}
#[test]
fn exact_iteration_visibility_matches_independent_outcomes()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted) in CASES {
        let text = source(name)?;
        rustc::qualify(
            &format!("parallel/pa26/iteration-visibility/source/{name}"),
            &text,
            accepted,
        )?;
        let result = frontend::checked(&text);
        assert_eq!(result.is_ok(), accepted, "{name}: {:?}", result.err());
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn exact_iteration_visibility_preserves_dispatch_and_borrows()
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
    native::check_literal_cases("parallel/pa26/iteration-visibility/native", &cases)
}
