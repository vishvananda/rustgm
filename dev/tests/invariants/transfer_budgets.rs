//! Exhausted cold recipe accounting cannot become a successful warm lookup.
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn nominal_schema_and_plan_reads_preserve_sticky_work_and_storage_failure() -> Result {
    let text =
        crate::test_source::fixture("pa31/tests/spec/unsizing-roles/nominal-coerce-required.rs")?;
    let report = crate::test_source::source_provider(&text, "transfer_budget")?;
    for storage in [false, true] {
        let mut facts = crate::sema::declarations::analyze(&report)?;
        let rules = facts
            .unsizing_rules
            .as_deref_mut()
            .ok_or("actual unsizing rules")?;
        let plans = rules
            .nominal
            .as_deref_mut()
            .ok_or("actual nominal recipes")?;
        let (&owner, schema) = plans.schemas.iter().next().ok_or("actual nominal schema")?;
        let (source, target) = (schema.source, schema.target);
        for _ in 0..8 {
            assert!(rules.nominal_schema(&facts.types, owner)?.is_some());
            assert!(rules.nominal_plan(&facts.types, source, target)?.is_none());
        }
        let plans = rules
            .nominal
            .as_deref_mut()
            .ok_or("retained recipe owner")?;
        assert!(
            if storage {
                plans.charge(0, usize::MAX)
            } else {
                plans.charge(usize::MAX, 0)
            }
            .is_err()
        );
        for _ in 0..8 {
            assert!(rules.nominal_schema(&facts.types, owner).is_err());
            assert!(rules.nominal_plan(&facts.types, source, target).is_err());
        }
    }
    Ok(())
}
