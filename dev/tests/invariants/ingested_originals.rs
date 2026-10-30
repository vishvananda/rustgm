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
        source.rows().originals.iter().any(retained_snapshot_read),
        "unchanged source retains its exact evaluated/materialized SSA read: {:?}",
        source.rows().originals,
    );
    let bytes = crates::encode(&artifact)?;
    assert_eq!(bytes.get(13..15), Some([9, 0].as_slice()));
    let report = crate::source_packets::client_report(crates::decode(&bytes)?, CLIENT)?;
    let facts = sema::declarations::analyze(&report)?;
    Ok((report, facts))
}
fn retained_snapshot_read(original: &s::Original) -> bool {
    original.values.iter().enumerate().any(|(index, value)| {
        let s::ValueKind::Read(s::Place::Temporary(snapshot)) = value.kind else {
            return false;
        };
        let Some(input) = original.values.get(snapshot as usize) else {
            return false;
        };
        let Ok(read) = u32::try_from(index) else {
            return false;
        };
        let evaluated = |id| {
            original
                .statements
                .iter()
                .enumerate()
                .filter_map(move |(position, statement)| {
                    (statement.action == s::Action::Evaluate(id)).then_some(position)
                })
        };
        let mut producer = evaluated(snapshot);
        let mut consumer = evaluated(read);
        matches!((producer.next(),consumer.next()), (Some(before),Some(after)) if before < after)
            && producer.next().is_none()
            && consumer.next().is_none()
            && (snapshot as usize) < index
            && input.ty == value.ty
            && original
                .temporaries
                .iter()
                .filter(|temporary| temporary.value == snapshot)
                .count()
                == 1
    })
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
fn actual_member_bindings(report: &Report, facts: &Declarations) -> Result {
    let mut count = 0;
    for (index, definition) in report.resolution.definitions.iter().enumerate() {
        if definition.kind != DefKind::Function || definition.node.is_some() {
            continue;
        }
        let id = DefId(u32::try_from(index)?);
        let Some(owner) = facts.associated.trait_owner(id) else {
            continue;
        };
        let scope = report.resolution.definitions[owner.0 as usize]
            .members
            .ok_or("actual imported trait scope")?;
        assert_eq!(definition.scope, scope);
        let binding = report
            .resolution
            .bindings
            .get(
                scope,
                definition.name,
                crate::resolve::model::Namespace::Value,
            )
            .ok_or("actual imported member binding")?;
        assert_eq!(binding.target, id);
        assert_eq!(binding.access, definition.access);
        count += 1;
    }
    assert!(count > 0);
    Ok(())
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
            .declaration_header(definition)?
            .is_some_and(|header| header.body.is_some())
        {
            result.push(definition);
        }
    }
    assert_eq!(result.len(), 2);
    Ok(result)
}
fn corrupt_late_header(rows: &mut s::SourceRows, opponent: usize) -> Result {
    let header = rows
        .functions
        .iter_mut()
        .find(|header| header.parent.is_some() && header.regions.total != 0)
        .ok_or("actual borrowed method header")?;
    let reference = *header
        .regions
        .late
        .slice(&rows.region_slots)?
        .first()
        .ok_or("actual original late slot")?;
    match opponent {
        0 => header.regions.total += 1,
        1 => header.regions.late.end -= 1,
        _ => {
            rows.slots[reference.0 as usize].owner =
                header.parent.ok_or("actual associated owner")?;
        }
    }
    Ok(())
}
fn corrupt_normalization(rows: &mut s::SourceRows, opponent: usize) -> Result {
    let (body, input) = rows
        .body_inputs
        .iter_mut()
        .find_map(|body| {
            body.normalizations
                .iter_mut()
                .find(|input| !input.owned.is_empty())
                .map(|input| (body.body, input))
        })
        .ok_or("actual body-owned normalization")?;
    let range = input.owned[0];
    match opponent {
        0 => input.owned.push(range),
        1 => input.owned[0].end = range.start,
        _ => {
            rows.originals[body.0 as usize].variables[range.start as usize].allocation =
                s::RegionAllocation::Inference {
                    needs_provenance: false,
                };
        }
    }
    Ok(())
}
fn applied_goal(rows: &s::SourceRows, implementation: bool) -> Result<s::TraitApplicationRef> {
    for call in rows.originals.iter().flat_map(|body| &body.calls) {
        let application = &rows.function_applications[call.target.0 as usize];
        let header = rows
            .functions
            .iter()
            .find(|header| header.declaration == application.declaration.declaration)
            .ok_or("actual target header")?;
        let Some(parent) = header.parent else {
            continue;
        };
        let Some(method) = call.method else { continue };
        if implementation {
            if rows.implementations.iter().any(|header| {
                header.self_definition == parent && header.trait_application != Some(method)
            }) {
                return Ok(method);
            }
        } else if rows
            .traits
            .iter()
            .any(|header| header.declaration == parent)
            && !rows.trait_applications[method.0 as usize]
                .arguments
                .slice(&rows.arguments)?
                .is_empty()
        {
            return Ok(method);
        }
    }
    Err("actual caller-applied method goal".into())
}
fn corrupt_applied_goal(rows: &mut s::SourceRows, opponent: usize) -> Result {
    let reference = applied_goal(rows, opponent < 3)?;
    let goal = &rows.trait_applications[reference.0 as usize];
    let other = rows
        .traits
        .iter()
        .find(|header| {
            header.written_parameters == 1 && header.declaration != goal.declaration.declaration
        })
        .ok_or("another actual trait declaration")?
        .declaration;
    let scalar = rows
        .types
        .iter()
        .position(|ty| matches!(ty, s::Type::Scalar(crate::support::crates::Scalar::F64)))
        .ok_or("actual distinct scalar type")?;
    let scalar = crate::support::crates::generics::TypeRef(u32::try_from(scalar)?);
    let goal = &mut rows.trait_applications[reference.0 as usize];
    match opponent % 3 {
        0 => goal.subject = scalar,
        1 => rows.arguments[goal.arguments.start as usize] = s::Argument::Type(scalar),
        _ => goal.declaration.declaration = other,
    }
    Ok(())
}
fn corrupt_member_header(
    rows: &mut s::SourceRows,
    root: crates::owner_catalog::DeclarationRef,
    opponent: usize,
) -> Result {
    let index = rows
        .functions
        .iter()
        .position(|header| {
            header
                .parent
                .is_some_and(|parent| rows.traits.iter().any(|owner| owner.declaration == parent))
        })
        .ok_or("actual trait member header")?;
    let header = &rows.functions[index];
    let foreign = rows
        .traits
        .iter()
        .find(|owner| Some(owner.declaration) != header.parent)
        .ok_or("another actual trait owner")?
        .declaration;
    match opponent {
        0 => rows.functions[index].parent = Some(root),
        1 => rows.functions[index].parent = Some(foreign),
        _ => rows.functions.push(rows.functions[index].clone()),
    }
    Ok(())
}
fn malformed_recorded_inputs() -> Result {
    let source = crate::test_source::fixture(PROVIDER)?;
    let artifact = crate::module_packets::exported_profile(
        &source,
        SourcePublication::ModuleConversions,
        "ordinary_recorded_inputs",
    )?;
    let descriptor = artifact.source_descriptor().ok_or("actual descriptor")?;
    for opponent in 0..20 {
        let (mut rows, attributes, modules) = crate::module_packets::raw(&artifact)?;
        if opponent < 3 {
            corrupt_late_header(&mut rows, opponent)?;
        } else if opponent < 6 {
            corrupt_normalization(&mut rows, opponent - 3)?;
        } else if opponent < 12 {
            corrupt_applied_goal(&mut rows, opponent - 6)?;
        } else if opponent >= 17 {
            corrupt_member_header(&mut rows, modules.namespace.root, opponent - 17)?;
        }
        if opponent == 16 {
            rows.clauses
                .first_mut()
                .ok_or("actual declaration input")?
                .requirement = s::Requirement::UnstableFeature("caller_only_permission".into());
        }
        let mut availability = descriptor.availability()?.clone();
        assert!(availability.policies.is_empty(), "actual no-Library source");
        match opponent {
            12 => {
                availability.declarations[0].declaration =
                    crate::support::crates::owner_catalog::DeclarationRef(u32::MAX);
            }
            13 => availability.declarations[0].constant = Some(s::AvailabilityRef(0)),
            14 => availability.bindings[0].regular = Some(s::AvailabilityRef(0)),
            15 => availability.declarations[0].regular = Some(s::AvailabilityRef(0)),
            _ => {}
        }
        let mut work = crate::support::crates::generics::Work::default();
        assert!(
            crate::support::crates::generics::Payload::from_source_module_conversions(
                artifact.identity(),
                rows,
                attributes,
                modules,
                availability,
                0,
                &mut work,
            )
            .is_err()
        );
        assert!(work.steps > 0);
    }
    Ok(())
}
fn staged_root_cannot_be_erased() -> Result {
    let source =
        crate::test_source::fixture("pa31/tests/spec/source-availability-transport/provider.rs")?;
    let artifact = crate::module_packets::exported_profile(
        &source,
        SourcePublication::ModuleAvailability,
        "staged_availability_root",
    )?;
    let descriptor = artifact
        .source_descriptor()
        .ok_or("actual staged descriptor")?;
    let (rows, attributes, modules) = crate::module_packets::raw(&artifact)?;
    let mut availability = descriptor.availability()?.clone();
    assert!(!availability.policies.is_empty());
    availability.declarations[modules.namespace.root.0 as usize].regular = None;
    let mut work = crate::support::crates::generics::Work::default();
    assert!(
        crate::support::crates::generics::Payload::from_source_module_conversions(
            artifact.identity(),
            rows,
            attributes,
            modules,
            availability,
            0,
            &mut work,
        )
        .is_err()
    );
    assert!(work.steps > 0);
    Ok(())
}
#[test]
fn ordinary_headers_do_not_allocate_unused_borrowed_original_domains() -> Result {
    malformed_recorded_inputs()?;
    staged_root_cannot_be_erased()?;
    let (report, mut facts) = fresh()?;
    actual_member_bindings(&report, &facts)?;
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
    assert!(facts.function_header(definitions[0]).is_err());
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
    assert_local_regions(&view, &facts, &mut steps)?;
    Ok(())
}
fn assert_local_regions(
    view: &SourceOriginal<'_>,
    facts: &Declarations,
    steps: &mut usize,
) -> Result {
    let rows = view.rows();
    let Some(s::HeaderRef::Function(index)) = rows.header(view.raw().declaration)? else {
        return Err("actual local original header".into());
    };
    let original_binder = rows.functions[index as usize].original.binder;
    let mut saw_borrow = false;
    for local in &view.raw().locals {
        let ty = view.ty(local.ty, steps)?;
        let Some(Kind::Reference { region, .. }) = facts.types.kind(ty) else {
            continue;
        };
        let Some(s::Type::Reference { region: raw, .. }) = rows.types.get(local.ty.0 as usize)
        else {
            return Err("actual local reference recipe".into());
        };
        assert_eq!(region, view.region(*raw, steps)?);
        assert_ne!(region, RegionId::STATIC);
        match rows.regions[raw.0 as usize] {
            s::Region::Bound {
                binder,
                depth,
                slot,
            } => {
                assert_eq!(binder, original_binder);
                assert_eq!(region.bound_parts(), Some((depth, u16::try_from(slot)?)));
            }
            s::Region::Variable { body, slot } => {
                assert_eq!(body, view.raw().body);
                assert_eq!(region, view.variable(slot, steps)?);
                assert!(region.bound_parts().is_none());
            }
            _ => return Err("local reference lost its actual bound or recorded domain".into()),
        }
        saw_borrow = true;
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
