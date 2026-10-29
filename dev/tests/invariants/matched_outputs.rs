//! Requested associated types pass the same complete result/method/constant kernels.
use super::*;
fn fixture(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/early-output-readiness/{name}.rs"
    ))?)
}
fn checked(name: &str) -> Result<crate::resolve::expansion::Report> {
    Ok(crate::test_source::source_provider(
        &fixture(name)?,
        "early_outputs",
    )?)
}
#[test]
fn actual_associated_outputs_complete_with_methods_constants_and_strict_old_seals() -> Result {
    let report = checked("source")?;
    let mut checker = Checker::new(&report)?;
    checker
        .facts
        .set_constant_executor(crate::pipeline::execution::Context::default().executor());
    checker.run()?;
    assert!(
        checker
            .facts
            .array_lengths
            .values()
            .any(|&bound| bound == 7)
    );
    let owner = *owners(&checker, "Produced")?
        .first()
        .ok_or("actual output impl")?;
    let mut pending = Pending::default();
    let mut plan = plan(&mut checker, &mut pending, owner)?;
    assert!(plan.has_types());
    assert!(!plan.has_constants());
    assert!(plan.require_no_associated(&checker.facts, owner).is_err());
    assert!(plan.require_constants(&checker.facts, owner).is_err());
    plan.require_complete(&checker.facts, owner)?;
    let removed = plan.members.pop().ok_or("actual complete bucket")?;
    assert!(plan.require_complete(&checker.facts, owner).is_err());
    plan.members.push(removed);
    plan.require_complete(&checker.facts, owner)?;
    let mut foreign = Checker::new(&report)?;
    foreign
        .facts
        .set_constant_executor(crate::pipeline::execution::Context::default().executor());
    foreign.run()?;
    assert!(plan.require_complete(&foreign.facts, owner).is_err());
    Ok(())
}
#[test]
fn missing_extra_changed_output_and_ordinary_impls_do_not_authorize_const_execution() -> Result {
    for name in [
        "wrong-bound",
        "missing-type",
        "extra-type",
        "wrong-method",
        "ordinary",
    ] {
        let report = checked(name)?;
        let mut checker = Checker::new(&report)?;
        checker
            .facts
            .set_constant_executor(crate::pipeline::execution::Context::default().executor());
        let error = checker
            .run()
            .err()
            .ok_or_else(|| format!("accepted {name}"))?;
        assert!(
            !error.message.contains("outside the early scalar profile"),
            "{name}: {error}"
        );
        assert!(
            !error.message.contains("source type index"),
            "{name}: {error}"
        );
        println!("early-output-negative {name}: {error}");
    }
    Ok(())
}
#[test]
fn source_type_indexes_revalidate_owner_content_and_failed_warm_payments() -> Result {
    let report = checked("source")?;
    let mut checker = Checker::new(&report)?;
    checker
        .facts
        .set_constant_executor(crate::pipeline::execution::Context::default().executor());
    checker.run()?;
    let owner = *owners(&checker, "Produced")?
        .first()
        .ok_or("actual output impl")?;
    let mut pending = Pending::default();
    let plan = plan(&mut checker, &mut pending, owner)?;
    let mut types = super::super::super::super::associated_types::Members::default();
    for actual in plan.require_complete(&checker.facts, owner)? {
        types.prepare_source_owner(&report, &checker.facts, actual)?;
    }
    let before = types.indexed_storage();
    for _ in 0..128 {
        types.prepare_source_owner(&report, &checker.facts, owner)?;
    }
    assert_eq!(types.indexed_storage(), before);
    let foreign = crate::test_source::source_provider(&fixture("source")?, "foreign_outputs")?;
    assert!(
        types
            .prepare_source_owner(&foreign, &checker.facts, owner)
            .is_err()
    );
    let source = plan.source;
    let values = checker.trait_impl_types(owner, source, plan.predicate, &types)?;
    let &(_, actual) = values.first().ok_or("actual associated type result")?;
    let old = checker.facts.definitions[actual.0 as usize].replace(
        crate::types::TypeId::primitive(crate::types::Primitive::Bool),
    );
    assert!(
        checker
            .trait_impl_types(owner, source, plan.predicate, &types)
            .is_err()
    );
    checker.facts.definitions[actual.0 as usize] = old;
    let mut fresh = super::super::super::super::associated_types::Members::default();
    assert!(
        crate::sema::header_attributes::charge_source_contract_query(
            &checker.facts,
            crate::support::crates::MAX_COMPONENTS
        )
        .is_err()
    );
    assert!(
        fresh
            .prepare_source_owner(&report, &checker.facts, owner)
            .is_err()
    );
    assert_eq!(fresh.indexed_storage(), (0, 0));
    assert!(
        types
            .prepare_source_owner(&report, &checker.facts, owner)
            .is_err()
    );
    assert_eq!(types.indexed_storage(), before);
    Ok(())
}
#[test]
fn associated_type_member_width_visits_only_requested_source_buckets() -> Result {
    use std::fmt::Write as _;
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut text = source("runtime-read")?;
        text.push_str("trait Wide:Sized{");
        for n in 0..width {
            write!(text, "type A{n};")?;
        }
        text.push_str("}impl Wide for u8{");
        for n in 0..width {
            write!(text, "type A{n}=u8;")?;
        }
        text.push('}');
        let report = crate::test_source::source_provider(&text, "output_width")?;
        let mut checker = Checker::new(&report)?;
        checker.run()?;
        let owner = *owners(&checker, "Wide")?
            .first()
            .ok_or("actual wide owner")?;
        let mut types = super::super::super::super::associated_types::Members::default();
        types.prepare_source_owner(&report, &checker.facts, owner)?;
        let (count, bytes) = types.indexed_storage();
        assert_eq!(count, 1);
        if let Some(old) = previous {
            assert!(bytes <= old * 5);
        }
        previous = Some(bytes);
        assert_eq!(std::mem::size_of::<crate::body::ir::Body>(), 456);
        println!("early-output-width {width} storage={bytes}");
    }
    Ok(())
}
