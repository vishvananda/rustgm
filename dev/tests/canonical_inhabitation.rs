//! Canonical generic fields preserve the existing contextual coverage domain.
use std::path::Path;
type TestResult = Result<(), Box<dyn std::error::Error>>;
#[test]
fn canonical_fields_and_visibility_control_empty_match_coverage() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().ok_or("missing repository root")?;
    for (path, accepted, matches) in [
        ("pa18/tests/spec/canonical-inhabitation/generic-visible-empty.rs", true, 1),
        ("pa18/tests/spec/canonical-inhabitation/generic-private-owner.rs", true, 1),
        ("pa18/tests/spec/canonical-inhabitation/generic-private-caller.rs", false, 1),
        ("pa18/tests/spec/canonical-inhabitation/generic-public-caller.rs", true, 1),
        ("pa13/tests/spec/canonical-inhabitation/private-caller.rs", false, 1),
        ("pa13/tests/spec/canonical-inhabitation/private-owner.rs", true, 1),
        ("pa18/tests/spec/canonical-inhabitation/generic-visibility-query.rs", true, 0),
    ] {
        let source = std::fs::read_to_string(root.join(path))?;
        let result = crate::attribute_contracts::analyze(&source);
        if accepted {
            let (_, facts) = result.map_err(|error| format!("{path}: {error}"))?;
            assert_eq!(facts.patterns.decisions.len(), matches, "{path}");
            assert!(facts.patterns.decisions.iter().all(|decision| decision.uninhabited), "{path}");
        } else {
            let error = result.err().ok_or_else(|| format!("{path} accepted"))?;
            assert!(error.to_string().contains("non-exhaustive match"), "{path}: {error}");
        }
    }
    Ok(())
}
