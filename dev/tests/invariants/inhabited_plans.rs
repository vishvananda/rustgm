//! Shared native properties retain source authority and pay attempted work on warm hits.
use super::*;
use crate::{body, sema::header_attributes, support::bindings::LangItem};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const SOURCE: &str = "#![feature(no_core,lang_items,intrinsics,rustc_attrs)]#![no_core]#[lang=\"pointee_sized\"]trait PointeeSized{}#[lang=\"meta_sized\"]trait MetaSized:PointeeSized{}#[lang=\"sized\"]trait Sized:MetaSized{}";

#[test]
fn warm_native_properties_pay_queries_without_reallocating_or_accepting_foreign_reports() -> Result
{
    let source = SOURCE.to_owned()
        + "#[rustc_intrinsic]const fn size_of<T>()->usize;fn caller()->usize{size_of::<u32>()}";
    let report = crate::test_source::source_provider(&source, "native_properties")?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let mut facts = crate::sema::bodies::analyze(&report, facts)?;
    let bodies = body::build(&report, &facts)?;
    let body = bodies.bodies.first().ok_or("actual caller")?;
    let callee = body
        .values
        .iter()
        .find_map(|value| match value.kind {
            body::ir::Rvalue::Call { callee, .. } => Some(callee),
            _ => None,
        })
        .ok_or("actual checked call")?;
    let instance = Instance::new(&facts.types, body.values[callee.0 as usize].ty)?;
    assert!(
        !facts
            .source_bindings
            .as_deref()
            .ok_or("catalog")?
            .has_runtime_layouts()
    );
    let ready = prepare(&report, &mut facts, instance, body.source)?;
    assert_eq!(ready.bits, 4);
    assert!(ready.assertion.is_none());
    let cold = header_attributes::source_contract_storage(&facts).ok_or("paid cold plan")?;
    for _ in 0..128 {
        assert_eq!(prepare(&report, &mut facts, instance, body.source)?.bits, 4);
    }
    let warm = header_attributes::source_contract_storage(&facts).ok_or("paid warm plan")?;
    assert_eq!(warm.0 - cold.0, 128);
    assert_eq!(warm.1, cold.1);
    let foreign = crate::test_source::source_provider(&source, "foreign_properties")?;
    assert!(prepare(&foreign, &mut facts, instance, body.source).is_err());
    assert!(
        header_attributes::charge_source_contract_query(
            &facts,
            crate::support::crates::MAX_COMPONENTS
        )
        .is_err()
    );
    let failed = header_attributes::source_contract_storage(&facts).ok_or("failed payer")?;
    assert!(prepare(&report, &mut facts, instance, body.source).is_err());
    let retried = header_attributes::source_contract_storage(&facts).ok_or("retried payer")?;
    assert!(retried.0 > failed.0);
    assert_eq!(retried.1, failed.1);
    Ok(())
}

#[test]
fn actual_nounwind_hook_requires_its_current_source_body_and_completed_signature() -> Result {
    let source =
        SOURCE.to_owned() + "#[lang=\"panic_nounwind\"]fn renamed(_: &'static str)->!{loop{}}";
    let report = crate::test_source::source_provider(&source, "native_hook")?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let declaration = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .language(LangItem::PanicNounwind)
        .ok_or("actual role")?;
    let entry = nounwind::prepare(&report, &mut facts)?;
    assert!(
        matches!(facts.types.kind(entry.instance.ty()), Some(Kind::FunctionItem { item, .. }) if item.0 == declaration.definition.0)
    );
    let foreign = crate::test_source::source_provider(&source, "foreign_hook")?;
    assert!(nounwind::prepare(&foreign, &mut facts).is_err());
    let index = facts.function_indices[&declaration.definition];
    let original = facts.functions[index];
    facts.functions[index].signature = TypeId::UNIT;
    assert!(nounwind::prepare(&report, &mut facts).is_err());
    facts.functions[index] = original;
    facts.functions[index].body = None;
    assert!(nounwind::prepare(&report, &mut facts).is_err());
    facts.functions[index] = original;
    assert_eq!(
        nounwind::prepare(&report, &mut facts)?.instance,
        entry.instance
    );
    Ok(())
}
