//! Actual serialized owners authenticate both cold and cached declared promises.
use crate::{
    lower::instances::SourcePublication,
    module_packets,
    resolve::{body_origin::DeclarationOrigin, model::DefId},
    sema::{model::Declarations, traits::results},
    support::{
        bindings::LangItem,
        crates::{
            self,
            generics::{self as g, source as s},
        },
    },
    types::{
        GenericArgs, ItemId, Kind, Primitive, TypeId,
        opaque::OpaqueId,
        parameters::{clauses::Clause, predicates::Bound},
    },
};
use std::sync::Arc;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn artifact() -> Result<crates::Interface> {
    module_packets::exported_profile(
        &crate::test_source::fixture("pa43/tests/spec/deferred-compiler-op/provider.rs")?,
        SourcePublication::CompilerOperations,
        "symbolic_contracts",
    )
}
fn fresh(bytes: &[u8]) -> Result<Declarations> {
    let source = crate::test_source::fixture("pa43/tests/spec/deferred-compiler-op/consumer.rs")?
        .replace("minicore", "base");
    let mut facts = crate::source_packets::client_analysis(crates::decode(bytes)?, &source)?
        .1
        .facts;
    // Complete the same genuine immutable headers again with no additions:
    // the existing epoch transition discards rules, giving a real cold query.
    let contracts = facts
        .generics
        .traits
        .as_deref_mut()
        .ok_or("actual contracts")?;
    contracts.complete_bindings(&facts.types, Vec::new())?;
    contracts.complete_results();
    Ok(facts)
}
fn canonical(facts: &Declarations) -> Result<(OpaqueId, TypeId, ItemId)> {
    assert_eq!(facts.opaques.owners.len(), 1);
    let ids = facts
        .opaques
        .owners
        .values()
        .next()
        .ok_or("actual opaque owner")?;
    assert_eq!(ids.len(), 1);
    let declaration = facts
        .opaques
        .declarations
        .get(&ids[0])
        .ok_or("actual declaration")?;
    let future = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .validated_role(LangItem::Future)?
        .ok_or("actual Future")?
        .definition;
    Ok((ids[0], declaration.application, ItemId(future.0)))
}
type Mutation = fn(&mut s::SourceRows);
fn validate(interface: &crates::Interface, rows: s::SourceRows) -> Result {
    let descriptor = interface.source_descriptor().ok_or("descriptor")?;
    let (_, attributes, modules) = module_packets::raw(interface)?;
    g::Payload::from_source_compiler_operations(
        interface.identity(),
        rows,
        attributes,
        modules,
        descriptor.availability()?.clone(),
        interface.base_retained_bytes(),
        &mut g::Work::default(),
    )?;
    Ok(())
}
#[test]
fn serialized_recipes_reject_ordinal_capture_schema_projection_and_body_forgery() -> Result {
    let interface = artifact()?;
    let (rows, _, _) = module_packets::raw(&interface)?;
    validate(&interface, rows.clone())?;
    assert_eq!(
        rows.opaque_futures[0].captures.end - rows.opaque_futures[0].captures.start,
        2
    );
    assert!((rows.opaque_futures[0].captures.end as usize) < rows.arguments.len());
    assert_eq!(
        rows.functions[0].declaration,
        rows.compiler_operations[0].declaration
    );
    let mutations: [(&str, Mutation); 14] = [
        ("duplicate ordinary owner", |rows| {
            rows.originals.push(rows.originals[0].clone());
        }),
        ("missing ordinary owner", |rows| {
            rows.originals.pop();
        }),
        ("operation plus ordinary", |rows| {
            let mut original = rows.originals[0].clone();
            original.body = rows.compiler_operations[0].body;
            original.declaration = rows.compiler_operations[0].declaration;
            rows.originals.push(original);
        }),
        ("opaque ordinal gap", |rows| {
            rows.opaque_futures[0].ordinal = 1;
        }),
        ("foreign opaque owner", |rows| {
            rows.opaque_futures[0].declaration.0 = u32::MAX;
        }),
        ("capture omission", |rows| {
            rows.opaque_futures[0].captures.end -= 1;
        }),
        ("capture expansion", |rows| {
            rows.opaque_futures[0].captures.end += 1;
        }),
        ("foreign callable capture frame", |rows| {
            if let s::Argument::Region(region) =
                rows.arguments[rows.opaque_futures[0].captures.start as usize]
                && let s::Region::Bound { binder, .. } = &mut rows.regions[region.0 as usize]
            {
                *binder = rows.functions[0].callable.binder;
            }
        }),
        ("wrong capture depth", |rows| {
            if let s::Argument::Region(region) =
                rows.arguments[rows.opaque_futures[0].captures.start as usize]
                && let s::Region::Bound { depth, .. } = &mut rows.regions[region.0 as usize]
            {
                *depth = 1;
            }
        }),
        ("invented early capture", |rows| {
            rows.functions[0].regions.early = rows.functions[0].regions.late;
        }),
        ("forged schema promise", |rows| {
            rows.associated_types[0].sized = !rows.associated_types[0].sized;
        }),
        ("foreign Output parent", |rows| {
            rows.associated_types[0].parent = rows.compiler_operations[0].declaration;
        }),
        ("foreign projection Self", |rows| {
            rows.projections[0].subject = rows.compiler_operations[0].output;
        }),
        ("unpaid family args", |rows| {
            rows.projections[0].arguments = rows.opaque_futures[0].captures;
        }),
    ];
    for (name, change) in mutations {
        let mut changed = rows.clone();
        change(&mut changed);
        assert!(validate(&interface, changed).is_err(), "accepted {name}");
    }
    Ok(())
}
#[test]
fn cold_and_warm_result_queries_reject_changed_actual_owner_and_promises() -> Result {
    let bytes = crates::encode(&artifact()?)?;
    let foreign = fresh(&bytes)?;
    let (foreign_id, _, _) = canonical(&foreign)?;
    let foreign_origin = foreign.opaques.declarations[&foreign_id].origin;
    for opponent in 0..7 {
        for warm_cache in [false, true] {
            let mut facts = fresh(&bytes)?;
            let (id, application, future) = canonical(&facts)?;
            if warm_cache {
                let (first, _) =
                    results::traits(&mut facts, application, future)?.ok_or("cold Future rules")?;
                let count = facts.types.len();
                for _ in 0..32 {
                    let (warm, steps) = results::traits(&mut facts, application, future)?
                        .ok_or("warm Future rules")?;
                    assert!(Arc::ptr_eq(&first, &warm));
                    assert_eq!(steps, 1);
                    assert_eq!(facts.types.len(), count);
                }
            }
            mutate_current(&mut facts, id, opponent, foreign_origin)?;
            assert!(
                facts.opaques.declarations[&id]
                    .require_current(&facts, id)
                    .is_err(),
                "accepted current opponent {opponent}, warm={warm_cache}"
            );
            if opponent != 6 {
                assert!(
                    results::traits(&mut facts, application, future).is_err(),
                    "rules accepted opponent {opponent}, warm={warm_cache}"
                );
            }
        }
    }
    Ok(())
}
fn mutate_current(
    facts: &mut Declarations,
    id: OpaqueId,
    opponent: u8,
    foreign: DeclarationOrigin,
) -> Result {
    match opponent {
        0 => {
            let Clause::Predicate(bound) = &mut facts
                .opaques
                .declarations
                .get_mut(&id)
                .ok_or("declaration")?
                .bounds[1]
            else {
                return Err("actual Output equality".into());
            };
            bound.bound = Bound::ProjectionEqual(TypeId::primitive(Primitive::U8));
        }
        1 => {
            facts
                .opaques
                .declarations
                .get_mut(&id)
                .ok_or("declaration")?
                .owner = DefId(u32::MAX);
        }
        2 => {
            let application = facts.types.intern(Kind::Opaque(id, GenericArgs::EMPTY))?;
            facts
                .opaques
                .declarations
                .get_mut(&id)
                .ok_or("declaration")?
                .application = application;
        }
        3 => facts.opaques.owners.clear(),
        4 => {
            facts
                .opaques
                .declarations
                .get_mut(&id)
                .ok_or("declaration")?
                .origin = foreign;
        }
        5 => {
            let source = facts
                .functions
                .first()
                .ok_or("actual client function")?
                .node;
            facts
                .opaques
                .declarations
                .get_mut(&id)
                .ok_or("declaration")?
                .origin = source.into();
        }
        6 => facts.types = crate::types::Store::default(),
        _ => return Err("unknown current-owner opponent".into()),
    }
    Ok(())
}
#[test]
fn actual_symbolic_body_cannot_supply_an_executable_original() -> Result {
    let bytes = crates::encode(&artifact()?)?;
    let facts = fresh(&bytes)?;
    let (id, _, _) = canonical(&facts)?;
    let owner = facts.opaques.declarations[&id].owner;
    let mut steps = 0;
    let error = crate::sema::crates::source_original(&facts, owner, &mut steps)
        .err()
        .ok_or("symbolic operation supplied an original")?;
    assert!(
        error.contains("execution requires genuine future state ownership"),
        "{error}"
    );
    assert!(steps > 0);
    assert_eq!(size_of::<crate::types::Kind>(), 24);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    println!(
        "actual cold trait Header={} resultInput={} resultElement={} opaqueDeclaration={}",
        size_of::<crate::sema::traits::Header>(),
        size_of::<crate::sema::declaration_inputs::Input>(),
        size_of::<(ItemId, crate::sema::declaration_inputs::Input)>(),
        size_of::<super::Declaration>()
    );
    Ok(())
}
