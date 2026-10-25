//! Real imported empty scopes retain closed callable identity and strict selection.
use super::*;
use crate::{source_packets, support::crates, types::ItemId};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn callable(
    facts: &Declarations,
    report: &crate::resolve::expansion::Report,
    name: &str,
) -> Result<(ItemId, TypeId, Instance)> {
    let definition = report
        .resolution
        .crates
        .as_ref()
        .ok_or("actual imported inventory")?
        .definitions()
        .find_map(|(external, definition)| {
            matches!(external.path(), [actual] if actual == name).then_some(definition)
        })
        .ok_or("actual imported callable")?;
    assert!(
        report.resolution.definitions[definition.0 as usize]
            .node
            .is_none()
    );
    let ty = facts.definitions[definition.0 as usize].ok_or("actual callable cell")?;
    let Some(Kind::FunctionItem {
        item, signature, ..
    }) = facts.types.kind(ty)
    else {
        return Err("actual imported definition is not a callable".into());
    };
    assert_eq!(item, ItemId(definition.0));
    Ok((item, signature, Instance::new(&facts.types, ty)?))
}

#[test]
fn real_empty_imported_scopes_use_closed_callable_registration_and_reject_wrong_records() -> Result
{
    let mut source = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
    )?;
    source.push_str(
        "pub fn keep(value:u8)->u8{value}pub fn discard(value:u8){}\
         pub fn unused<T>(value:u8)->u8{value}",
    );
    let interface = source_packets::exported(&source)?;
    let bytes = crates::encode(&interface)?;
    let decoded = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&decoded)?, bytes);
    let (report, mut analysis, _) = source_packets::client_analysis(
        decoded,
        "#![feature(no_core)]#![no_core]extern crate base;pub fn idle(){}",
    )?;
    let facts = &mut analysis.facts;
    let (keep, signature, instance) = callable(facts, &report, "keep")?;
    let (discard, other_signature, _) = callable(facts, &report, "discard")?;
    let (unused, _, _) = callable(facts, &report, "unused")?;
    assert_ne!(signature, other_signature);
    for item in [keep, discard] {
        assert!(
            facts
                .generics
                .parameters
                .get(&DefId(item.0))
                .is_some_and(Vec::is_empty)
        );
        assert!(facts.generics.has_parameters(DefId(item.0)));
        let template = facts
            .lifetimes
            .callables
            .get(item)
            .ok_or("closed template")?;
        assert_eq!(template.parameters, 0);
        assert_eq!(
            facts.lifetimes.callables.generic_parameters(item),
            Some(&[][..])
        );
    }
    assert_eq!(
        facts
            .lifetimes
            .callables
            .get(keep)
            .ok_or("keep template")?
            .signature,
        signature
    );
    assert_eq!(
        facts
            .generics
            .parameters
            .get(&DefId(unused.0))
            .ok_or("unused formal")?
            .len(),
        1
    );
    assert_eq!(
        facts
            .lifetimes
            .callables
            .generic_parameters(unused)
            .ok_or("unused callable formal")?
            .len(),
        1
    );
    let selected = select(facts, instance)?;
    assert!(selected.bindings.is_empty() && selected.captures.is_empty());
    assert!(selected.comparison.is_none());
    wrong_records(facts, keep, discard, signature, other_signature, instance)?;
    Ok(())
}

fn wrong_records(
    facts: &mut Declarations,
    keep: ItemId,
    discard: ItemId,
    signature: TypeId,
    other_signature: TypeId,
    instance: Instance,
) -> Result {
    let original = std::mem::take(&mut facts.lifetimes.callables);
    facts
        .lifetimes
        .callables
        .insert_monomorphic(&facts.types, keep, other_signature)?;
    assert_eq!(
        select(facts, instance).err(),
        Some("instance signature differs from substituted template")
    );
    facts.lifetimes.callables = crate::types::callable::Templates::default();
    facts
        .lifetimes
        .callables
        .insert_monomorphic(&facts.types, discard, signature)?;
    assert_eq!(
        select(facts, instance).err(),
        Some("missing callable template")
    );
    facts.lifetimes.callables = crate::types::callable::Templates::default();
    assert_eq!(
        select(facts, instance).err(),
        Some("missing callable template")
    );
    facts.lifetimes.callables = original;
    assert!(select(facts, instance)?.bindings.is_empty());
    Ok(())
}
