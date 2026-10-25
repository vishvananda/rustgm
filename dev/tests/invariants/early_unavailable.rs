//! The unsupported diagnostic follows the actual source scheduler's retained decision.
use super::*;

#[test]
fn generic_prerequisite_publishes_only_exact_unvalidated_profile_evidence() -> Result {
    let report = crate::test_source::source_provider(
        &source("generic-clone-prerequisite")?,
        "source_traits_v1",
    )?;
    let mut checker = Checker::new(&report)?;
    checker
        .facts
        .set_constant_executor(crate::pipeline::execution::Context::default().executor());
    let error = checker
        .run()
        .err()
        .ok_or("generic early prerequisite unexpectedly supported")?;
    assert!(
        error
            .message
            .contains("implementation validation is outside the early scalar profile"),
        "{error}"
    );
    let copy = *owners(&checker, "Copy")?
        .first()
        .ok_or("actual Copy owner")?;
    let clone = *owners(&checker, "Clone")?
        .first()
        .ok_or("actual generic Clone owner")?;
    let pending = checker
        .early_implementations
        .as_ref()
        .ok_or("actual scheduler")?;
    assert!(matches!(
        pending.skipped.get(&clone),
        Some(Unsupported::OpenSubject)
    ));
    assert!(matches!(
        pending.skipped.get(&copy),
        Some(Unsupported::Requirements)
    ));
    let registry = checker
        .facts
        .generics
        .implementations
        .as_ref()
        .ok_or("actual registry")?;
    for owner in [clone, copy] {
        assert!(registry.early_unavailable(&checker.facts.types, owner)?);
        assert!(!registry.is_validated(owner));
        let header = registry.header(owner).ok_or("actual source header")?;
        let source = checker
            .facts
            .associated
            .owner(owner)
            .and_then(|id| checker.facts.associated.get(id))
            .ok_or("actual source owner")?;
        assert_eq!(source.source, header.source);
        assert_eq!(source.self_definition, header.owner);
    }
    println!(
        "generic early profile: Copy={copy:?}/Requirements Clone={clone:?}/OpenSubject; both remain unvalidated"
    );
    Ok(())
}
