//! Default eligibility keeps written projection lifetimes separate from normalized uses.
use super::{object_outcome, semantic};
const CASES: &[(&str, bool)] = &[
    ("projection-static-argument", true),
    ("projection-static-argument-used", false),
    ("projection-static-subject", true),
    ("projection-static-subject-used", false),
    ("projection-erased-subject", true),
    ("projection-erased-subject-used", false),
    ("introduced-static-result", false),
    ("introduced-callable-result", false),
    ("introduced-static-result-satisfied", true),
    ("introduced-callable-result-satisfied", true),
];

#[test]
fn projected_defaults_preserve_original_lifetime_classification()
-> Result<(), Box<dyn std::error::Error>> {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa20/tests/spec/default-requirements");
    for &(name, accepted) in CASES {
        let source = std::fs::read_to_string(folder.join(format!("{name}.rs")))?;
        object_outcome(&format!("default-{name}"), &source, accepted)?;
        let result = semantic::analyze(&source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}
