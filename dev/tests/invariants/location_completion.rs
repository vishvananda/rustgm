//! Reentry revalidates genuine source data before consulting nominal completion.
use super::*;
use crate::{
    sema::compiler_items::{phantom, roles},
    test_source,
    types::aggregate,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const SOURCE: &str = "pa31/tests/spec/source-location/location-copy-clone.rs";
fn report(name: &str) -> Result<Report, Box<dyn std::error::Error>> {
    Ok(test_source::source_provider(
        &test_source::fixture(SOURCE)?,
        name,
    )?)
}
fn declaration(
    facts: &Declarations,
    role: LangItem,
) -> Result<Declaration, Box<dyn std::error::Error>> {
    Ok(facts
        .source_bindings
        .as_deref()
        .and_then(|c| c.language(role))
        .ok_or("actual nominal")?)
}
#[test]
fn location_and_phantom_reentry_revalidate_before_reusing_actual_completion() -> TestResult {
    let report = report("location_reentry")?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let location = declaration(&facts, LangItem::PanicLocation)?;
    assert_eq!(requested_location(&report, &facts)?, None);
    let catalog = facts
        .source_bindings
        .as_deref_mut()
        .ok_or("source catalog")?;
    let ledger = catalog.traits.as_deref_mut().ok_or("actual completion")?;
    ledger.roles.remove(LangItem::PanicLocation);
    ledger.roles.remove(LangItem::PhantomData);
    assert_eq!(
        requested_location(&report, &facts)?,
        Some(location.definition)
    );
    assert!(complete_location(&report, &mut facts).is_err());
    phantom::complete(&report, &mut facts)?;
    complete_location(&report, &mut facts)?;
    for _ in 0..2 {
        phantom::complete(&report, &mut facts)?;
        complete_location(&report, &mut facts)?;
        complete(&report, &mut facts)?;
    }
    assert_eq!(requested_location(&report, &facts)?, None);
    assert!(roles::complete_nominal(&mut facts, LangItem::PanicLocation, location).is_err());
    let marker = declaration(&facts, LangItem::PhantomData)?;
    assert!(roles::complete_nominal(&mut facts, LangItem::PhantomData, marker).is_err());
    assert!(
        facts
            .source_bindings
            .as_deref()
            .ok_or("catalog")?
            .language(LangItem::PanicInfo)
            .is_none()
    );
    Ok(())
}
#[test]
fn completed_nominals_still_reject_changed_types_fields_and_marker_category() -> TestResult {
    let report = report("location_mutation")?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let location = declaration(&facts, LangItem::PanicLocation)?;
    let index = location.definition.0 as usize;
    let original = facts.definitions[index];
    facts.definitions[index] = Some(TypeId::primitive(Primitive::U32));
    assert!(complete_location(&report, &mut facts).is_err());
    facts.definitions[index] = original;
    let item = ItemId(location.definition.0);
    let mut changed = fields(&facts, location)?.to_vec();
    changed[1] = TypeId::primitive(Primitive::U64);
    let changed = facts.types.intern_list(&changed)?;
    let original = std::mem::take(&mut facts.aggregates);
    facts.aggregates.insert(
        item,
        aggregate::Kind::Struct,
        aggregate::Repr::default(),
        &[aggregate::Variant {
            fields: changed,
            discriminant: 0,
        }],
    )?;
    assert!(complete_location(&report, &mut facts).is_err());
    facts.aggregates = original;
    complete_location(&report, &mut facts)?;
    let marker = declaration(&facts, LangItem::PhantomData)?;
    let original = std::mem::take(&mut facts.aggregates);
    let empty = facts.types.intern_list(&[])?;
    facts.aggregates.insert(
        ItemId(marker.definition.0),
        aggregate::Kind::Struct,
        aggregate::Repr::default(),
        &[aggregate::Variant {
            fields: empty,
            discriminant: 0,
        }],
    )?;
    assert!(phantom::complete(&report, &mut facts).is_err());
    facts.aggregates = original;
    phantom::complete(&report, &mut facts)?;
    Ok(())
}
#[test]
fn nominal_readiness_rejects_foreign_identity_wrong_roles_and_failed_payment() -> TestResult {
    let report = report("location_identity")?;
    let foreign = self::report("location_foreign")?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let location = declaration(&facts, LangItem::PanicLocation)?;
    assert!(requested_location(&foreign, &facts).is_err());
    assert!(complete_location(&foreign, &mut facts).is_err());
    let mut wrong = location;
    wrong.definition = crate::resolve::model::DefId(location.definition.0 + 1);
    assert!(roles::nominal_completed(&report, &facts, LangItem::PanicLocation, wrong).is_err());
    assert!(roles::nominal_completed(&report, &facts, LangItem::Clone, location).is_err());
    let catalog = facts.source_bindings.take();
    assert!(requested_location(&report, &facts).is_err());
    facts.source_bindings = catalog;
    facts
        .source_bindings
        .as_deref_mut()
        .and_then(|catalog| catalog.traits.as_deref_mut())
        .ok_or("actual completion ledger")?
        .roles
        .remove(LangItem::PanicLocation);
    assert!(
        crate::sema::header_attributes::charge_source_contract_query(
            &facts,
            crate::support::crates::MAX_COMPONENTS,
        )
        .is_err()
    );
    assert!(roles::nominal_completed(&report, &facts, LangItem::PanicLocation, location).is_err());
    assert!(requested_location(&report, &facts).is_err());
    assert!(complete_location(&report, &mut facts).is_err());
    assert!(
        facts
            .source_bindings
            .as_deref()
            .ok_or("catalog")?
            .validated_language(LangItem::PanicLocation)
            .is_err()
    );
    Ok(())
}
