//! Genuine v9 ordinary source declarations and demanded borrowed originals.
use super::*;
use crate::{
    lower::instances::SourcePublication,
    resolve::expansion::Report,
    sema,
    support::crates,
    types::{Kind, RegionId},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PROVIDER: &str = "pa31/tests/spec/source-from/nested-into-dot.rs";
const CLIENT: &str = "#![feature(no_core)]#![no_core]extern crate base;";

fn fresh() -> Result<(Report, Declarations)> {
    let source = crate::test_source::fixture(PROVIDER)?;
    let artifact = crate::module_packets::exported_profile(
        &source,
        SourcePublication::ModuleConversions,
        "ordinary_recorded",
    )?;
    let source = artifact
        .source_descriptor()
        .ok_or("actual source descriptor")?;
    assert!(
        source.rows().originals.iter().any(|original| {
            original
                .projections
                .iter()
                .any(|projection| matches!(projection.base, s::Place::Temporary(_)))
        }),
        "unchanged borrowed Clone bodies retain their actual SSA reference base"
    );
    let bytes = crates::encode(&artifact)?;
    assert_eq!(bytes.get(13..15), Some([9, 0].as_slice()));
    let report = crate::source_packets::client_report(crates::decode(&bytes)?, CLIENT)?;
    let facts = sema::declarations::analyze(&report)?;
    Ok((report, facts))
}
fn mapped_graph(facts: &Declarations) -> Result<Rc<Graph>> {
    let Some(crate::sema::crates::ImportedOwners::Ordinary(graph)) =
        facts.associated.imported_owner()
    else {
        return Err("actual ordinary source graph".into());
    };
    Ok(Rc::clone(graph))
}
fn observed(graph: &Graph) -> Result<(usize, usize, usize)> {
    let Canonical::Lazy(lazy) = &graph.canonical else {
        return Err("actual v9 lazy graph".into());
    };
    Ok(lazy.observed()?)
}
fn clones(report: &Report) -> Result<Vec<DefId>> {
    let mut result = Vec::new();
    for (index, definition) in report.resolution.definitions.iter().enumerate() {
        if definition.kind == DefKind::Function
            && definition.node.is_none()
            && report.package.names.get(definition.name) == "clone"
        {
            result.push(DefId(u32::try_from(index)?));
        }
    }
    assert!(
        result.len() >= 3,
        "required Clone declaration plus real f32/f64 bodies"
    );
    Ok(result)
}
fn bodies(facts: &Declarations, report: &Report) -> Result<Vec<DefId>> {
    let mut result = Vec::new();
    for definition in clones(report)? {
        if facts
            .function_header(definition)?
            .is_some_and(|header| header.body.is_some())
        {
            result.push(definition);
        }
    }
    assert_eq!(result.len(), 2);
    Ok(result)
}
#[test]
fn ordinary_headers_do_not_allocate_unused_borrowed_original_domains() -> Result {
    let (report, mut facts) = fresh()?;
    let graph = mapped_graph(&facts)?;
    assert_eq!(observed(&graph)?.0, 0);
    let definitions = bodies(&facts, &report)?;
    assert_eq!(
        facts
            .crates
            .as_deref()
            .and_then(|linkage| linkage.originals.as_deref())
            .and_then(|headers| headers.get(definitions[0]))
            .ok_or("header")?
            .universals,
        1
    );
    assert_eq!(
        super::super::source_readiness(&mut facts, definitions[0])?,
        Some(Readiness::Declaration)
    );
    assert!(crate::sema::crates::source_original(&facts, definitions[0], &mut 0).is_err());
    assert_eq!(observed(&graph)?.0, 0);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    println!(
        "ordinary graph={} lazy={} original={} body={} node={}",
        size_of::<Graph>(),
        size_of::<ingested_domains::Lazy>(),
        size_of::<SourceOriginal<'_>>(),
        size_of::<crate::body::ir::Body>(),
        size_of::<crate::syntax::ast::Node>()
    );
    Ok(())
}
#[test]
fn demanded_originals_use_actual_allocation_order_and_reject_foreign_body_regions() -> Result {
    let (report, mut facts) = fresh()?;
    let definitions = bodies(&facts, &report)?;
    crate::sema::crates::prepare_source_original(&report, &mut facts, definitions[0], 0, 0)?;
    let mut steps = 0;
    let view =
        crate::sema::crates::source_original(&facts, definitions[0], &mut steps)?.ok_or("view")?;
    assert!(view.numbered());
    assert!(!view.raw().variables.is_empty());
    let rows = view.rows();
    let unit = &rows.body_units[view.raw().body.0 as usize];
    let first = *unit
        .allocation_order
        .first()
        .ok_or("real first allocation")?;
    assert_eq!(view.variable(first, &mut steps)?, RegionId(1));
    let own = rows
        .regions
        .iter()
        .enumerate()
        .find_map(|(index, region)| {
            matches!(region, s::Region::Variable { body, .. } if *body == view.raw().body)
                .then_some(index)
        })
        .ok_or("actual owned region")?;
    let s::Region::Variable { slot, .. } = rows.regions[own] else {
        return Err("owned region".into());
    };
    assert_eq!(
        view.region(s::RegionRef(u32::try_from(own)?), &mut steps)?,
        view.variable(slot, &mut steps)?
    );
    let foreign = rows
        .regions
        .iter()
        .enumerate()
        .find_map(|(index, region)| {
            matches!(region, s::Region::Variable { body, .. } if *body != view.raw().body)
                .then_some(index)
        })
        .ok_or("real other body region")?;
    assert!(
        view.region(s::RegionRef(u32::try_from(foreign)?), &mut steps)
            .is_err()
    );
    let before = steps;
    assert!(view.variable(u32::MAX, &mut steps).is_err());
    assert!(steps > before);
    let mut saw_borrow = false;
    for local in &view.raw().locals {
        let ty = view.ty(local.ty, &mut steps)?;
        if let Some(Kind::Reference { region, .. }) = facts.types.kind(ty) {
            assert!(region != RegionId::STATIC && region.bound_parts().is_none());
            saw_borrow = true;
        }
    }
    assert!(saw_borrow);
    Ok(())
}
#[test]
fn ordinary_originals_revalidate_store_header_and_exact_constructed_environment() -> Result {
    let (report, mut facts) = fresh()?;
    let definitions = bodies(&facts, &report)?;
    let definition = definitions[0];
    crate::sema::crates::prepare_source_original(&report, &mut facts, definition, 0, 0)?;
    let store = std::mem::take(&mut facts.types);
    assert!(crate::sema::crates::source_original(&facts, definition, &mut 0).is_err());
    facts.types = store;
    let foreign = fresh()?.0;
    assert!(
        crate::sema::crates::prepare_source_original(&foreign, &mut facts, definition, 0, 0)
            .is_err()
    );

    let headers = facts
        .crates
        .as_deref_mut()
        .and_then(|linkage| linkage.originals.as_deref_mut())
        .ok_or("headers")?;
    let header = headers.entries.get_mut(&definition).ok_or("header")?;
    let signature = header.signature;
    header.signature = crate::types::TypeId::UNIT;
    assert!(crate::sema::crates::source_original(&facts, definition, &mut 0).is_err());
    let header = facts
        .crates
        .as_deref_mut()
        .and_then(|linkage| linkage.originals.as_deref_mut())
        .and_then(|headers| headers.entries.get_mut(&definition))
        .ok_or("header")?;
    header.signature = signature;
    let environment = header.environment.take().ok_or("constructed environment")?;
    header.environment = Some(Arc::new(environment.as_ref().clone()));
    assert!(crate::sema::crates::source_original(&facts, definition, &mut 0).is_err());
    facts
        .crates
        .as_deref_mut()
        .and_then(|linkage| linkage.originals.as_deref_mut())
        .and_then(|headers| headers.entries.get_mut(&definition))
        .ok_or("header")?
        .environment = Some(environment);
    assert!(crate::sema::crates::source_original(&facts, definition, &mut 0)?.is_some());
    Ok(())
}
#[test]
fn ordinary_original_warm_queries_reuse_memo_and_failed_prepayment_is_sticky() -> Result {
    let (report, mut facts) = fresh()?;
    let graph = mapped_graph(&facts)?;
    let definitions = bodies(&facts, &report)?;
    let definition = definitions[0];
    crate::sema::crates::prepare_source_original(&report, &mut facts, definition, 0, 0)?;
    let before = observed(&graph)?;
    let types = facts.types.len();
    let bytes = facts
        .crates
        .as_deref()
        .and_then(crate::sema::crates::Linkage::original_work)
        .ok_or("cost")?
        .1;
    for _ in 0..16 {
        crate::sema::crates::prepare_source_original(&report, &mut facts, definition, 0, 0)?;
    }
    assert_eq!(observed(&graph)?, before);
    assert_eq!(facts.types.len(), types);
    assert_eq!(
        facts
            .crates
            .as_deref()
            .and_then(crate::sema::crates::Linkage::original_work)
            .ok_or("cost")?
            .1,
        bytes
    );
    let headers = facts
        .crates
        .as_deref()
        .and_then(|linkage| linkage.originals.as_deref())
        .ok_or("headers")?;
    headers.work.set(crate::support::crates::MAX_COMPONENTS);
    for _ in 0..2 {
        assert!(
            crate::sema::crates::prepare_source_original(&report, &mut facts, definitions[1], 0, 0)
                .is_err()
        );
    }
    assert_eq!(observed(&graph)?, before);
    assert_eq!(facts.types.len(), types);
    assert!(crate::sema::crates::source_original(&facts, definition, &mut 0).is_err());
    let (report, mut facts) = fresh()?;
    let actual = mapped_graph(&facts)?;
    let definition = bodies(&facts, &report)?[0];
    let types = facts.types.len();
    facts
        .crates
        .as_deref_mut()
        .and_then(|linkage| linkage.originals.as_deref_mut())
        .ok_or("headers")?
        .bytes = crate::support::crates::MAX_BYTES;
    for _ in 0..2 {
        assert!(
            crate::sema::crates::prepare_source_original(&report, &mut facts, definition, 0, 0)
                .is_err()
        );
    }
    assert_eq!(observed(&actual)?.0, 0);
    assert_eq!(facts.types.len(), types);
    Ok(())
}
