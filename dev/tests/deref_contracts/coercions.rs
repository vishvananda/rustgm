//! Direct and explicit places anchor the separately frozen implicit conversions.
use super::{TestResult, program, test_source};

#[test]
fn direct_and_explicit_reference_controls_retain_checked_body_contracts() -> TestResult {
    for name in ["direct-known", "explicit-place", "explicit-temporary"] {
        let source = test_source::fixture(&format!("pa31/tests/spec/borrow-coercions/{name}.rs"))?;
        program(&source).map_err(|error| format!("{name}: {error}"))?;
    }
    for (name, cause) in [
        (
            "implicit-mutability-reject",
            "cannot coerce a shared reference to a mutable reference",
        ),
        ("implicit-wrong-target-reject", "incompatible types"),
    ] {
        let source = test_source::fixture(&format!("pa31/tests/spec/borrow-coercions/{name}.rs"))?;
        let error = program(&source)
            .err()
            .ok_or("accepted invalid conversion")?;
        assert!(error.to_string().contains(cause), "{name}: {error}");
    }
    // Seven implicit positives, the actual escape rejection, and native order
    // remain required in both manifests. These controls do not qualify them.
    Ok(())
}
