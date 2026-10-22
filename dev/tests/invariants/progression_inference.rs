//! A common canonical Item equation is never an implementation proof.
use super::*;
use crate::{sema, types};
use sema::{
    declarations::iteration::range_contracts,
    traits::{implementations::Availability, selection::Dependency},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn fixture() -> TestResult<Declarations> {
    let report = crate::test_source::resolve("fn main(){for _ in 1u8..4u8{}}", true)?;
    Ok(crate::pipeline::analyze_source(&report)?.facts)
}
#[test]
fn common_header_item_preserves_inputs_and_rejects_unpaid_or_wrong_families() -> TestResult {
    let mut facts = fixture()?;
    let originals = facts
        .iteration
        .as_deref()
        .and_then(|r| r.ranges.as_deref())
        .ok_or("registry")?
        .originals
        .clone();
    let original = originals[0];
    let types::Kind::Adt(nominal, _) = facts.types.kind(original.subject).ok_or("Range")? else {
        return Err("nominal".into());
    };
    let mut context = Context::default();
    let variable = context.variable(types::infer::Domain::Integer)?;
    let subject = context.nominal(
        nominal,
        &[types::infer::arguments::Argument::Type(variable)],
    )?;
    let result = range_element(&mut facts, &mut context, subject, None)?.ok_or("common Item")?;
    assert_eq!(context.root(result)?, context.root(variable)?);
    assert_eq!(
        context.unresolved_domain(variable)?,
        Some(types::infer::Domain::Integer)
    );
    let iterator = facts
        .iteration
        .as_deref()
        .ok_or("iteration")?
        .canonical
        .get(crate::support::iteration::Trait::Iterator);
    let mut assumptions = Assumptions::default();
    assumptions.insert(&facts.types, original.predicate)?;
    assert!(range_element(&mut facts, &mut context, subject, Some(&assumptions))?.is_none());
    let boolean = context.known(
        &facts.types,
        types::TypeId::primitive(types::Primitive::Bool),
    )?;
    let wrong = context.nominal(nominal, &[types::infer::arguments::Argument::Type(boolean)])?;
    assert!(range_element(&mut facts, &mut context, wrong, None)?.is_none());
    for pending in [false, true] {
        let inventory =
            competing_inventory(&mut facts, nominal, iterator.definition, original, pending)?;
        facts.generics.implementations = Some(Box::new(inventory));
        assert!(range_element(&mut facts, &mut context, subject, None)?.is_none());
    }
    Ok(())
}

fn competing_inventory(
    facts: &mut Declarations,
    nominal: types::ItemId,
    iterator: crate::resolve::model::DefId,
    original: crate::sema::iteration::range_originals::Original,
    pending: bool,
) -> TestResult<sema::traits::implementations::Implementations> {
    let originals = facts
        .iteration
        .as_deref()
        .and_then(|r| r.ranges.as_deref())
        .ok_or("registry")?
        .originals
        .clone();
    let rows = facts
        .generics
        .implementations
        .as_ref()
        .ok_or("inventory")?
        .headers()
        .filter(|h| {
            originals
                .iter()
                .any(|o| o.declaration.definition == h.owner)
        })
        .map(|h| {
            (
                h.owner,
                h.source,
                h.predicate,
                h.methods.clone(),
                h.types.clone(),
                h.constants.clone(),
            )
        })
        .collect::<Vec<_>>();
    let mut inventory = sema::traits::implementations::Implementations::default();
    let forged = crate::resolve::model::DefId(u32::MAX - 1);
    let mut reserved = rows
        .iter()
        .map(|row| sema::traits::implementations::PendingHeader {
            owner: row.0,
            item: types::ItemId(iterator.0).into(),
            head: Some(crate::types::subject_head::Head::Nominal(nominal)),
        })
        .collect::<Vec<_>>();
    reserved.push(sema::traits::implementations::PendingHeader {
        owner: forged,
        item: types::ItemId(iterator.0).into(),
        head: None,
    });
    inventory.reserve(reserved)?;
    // An open source competitor must prevent the relation, even in an
    // adversarial inventory that ordinary coherence would later reject.
    for (owner, source, predicate, methods, types, constants) in &rows {
        inventory.prepare(
            &mut facts.types,
            types::ItemId(iterator.0),
            sema::traits::implementations::Implementation {
                owner: *owner,
                source: *source,
                predicate: *predicate,
                methods: Vec::new(),
                types: Vec::new(),
                constants: Vec::new(),
            },
        )?;
        inventory.complete(*owner, methods.clone(), types.clone(), constants.clone())?;
    }
    if !pending {
        inventory.prepare(
            &mut facts.types,
            types::ItemId(iterator.0),
            sema::traits::implementations::Implementation {
                owner: forged,
                source: rows[0].1,
                predicate: original.predicate,
                methods: Vec::new(),
                types: Vec::new(),
                constants: Vec::new(),
            },
        )?;
        inventory.complete(forged, Vec::new(), Vec::new(), Vec::new())?;
    }
    Ok(inventory)
}

#[test]
fn range_header_retry_rejects_member_ids_and_completed_headers() -> TestResult {
    let report = crate::test_source::resolve("fn main(){for _ in 1u8..4u8{}}", true)?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let original = facts
        .iteration
        .as_deref()
        .and_then(|r| r.ranges.as_deref())
        .ok_or("registry")?
        .originals[0];
    for owner in [original.declaration.item, original.declaration.method] {
        assert!(!range_contracts::prepare_dependency(
            &report,
            &mut facts,
            Dependency::Header(owner),
            Availability::Validated
        )?);
    }
    assert!(
        range_contracts::prepare_dependency(
            &report,
            &mut facts,
            Dependency::Header(original.declaration.definition),
            Availability::Validated
        )
        .is_err()
    );
    Ok(())
}
