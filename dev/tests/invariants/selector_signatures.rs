//! Logical reached-call signatures are distinct from nested public signatures.
//! These tests exercise data validation and do not construct execution witnesses.
use super::*;
use crate::types::{ItemId, RegionId, Store, arguments::Argument};

fn signature(
    store: &mut Store,
    parameters: &[TypeId],
    result: TypeId,
) -> Result<TypeId, &'static str> {
    let parameters = store.intern_list(parameters)?;
    store.intern(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    })
}
fn reference(store: &mut Store, slot: u16) -> Result<TypeId, &'static str> {
    store.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(0, slot)?,
        referent: TypeId::primitive(crate::types::Primitive::U8),
    })
}

#[test]
fn opened_selector_requires_exact_logical_roots_including_unused_reference_binders()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let reference = reference(&mut store, 0)?;
    let parameters = store.intern_list(&[reference])?;
    let tuple = store.intern(Kind::Tuple(parameters))?;
    let callback = signature(&mut store, &[], TypeId::UNIT)?;
    let actual = store.intern_arguments(&[Argument::Type(reference)])?;
    let f = store.intern(Kind::FunctionItem {
        item: ItemId(10),
        arguments: actual,
        signature: callback,
    })?;
    let g = store.intern(Kind::FunctionItem {
        item: ItemId(11),
        arguments: actual,
        signature: callback,
    })?;
    let types = [tuple, f, g, reference];
    let mut public = [TypeId::UNIT; 4];
    for (output, input) in public.iter_mut().zip(types) {
        *output = store.shift_type(input, 1)?.0;
    }
    let public_signature = signature(&mut store, &public[..3], public[3])?;
    let opened = signature(&mut store, &types[..3], types[3])?;
    assert_ne!(public_signature, opened);
    let arguments = store.intern_arguments(&types.map(Argument::Type))?;
    let function = store.intern(Kind::FunctionItem {
        item: ItemId(12),
        arguments,
        signature: public_signature,
    })?;
    let selection = Select {
        definition: crate::resolve::model::DefId(12),
        instance: Instance::new(&store, function)?,
        signature: public_signature,
        types,
    };
    opened_signature(&store, opened, reference, selection)?;
    assert!(opened_signature(&store, public_signature, reference, selection).is_err());
    assert!(opened_signature(&store, opened, TypeId::UNIT, selection).is_err());
    let foreign = self::reference(&mut store, 1)?;
    let wrong = signature(&mut store, &[tuple, f, g], foreign)?;
    assert!(opened_signature(&store, wrong, foreign, selection).is_err());
    let Some(Kind::Function {
        parameters,
        result,
        unsafe_,
        convention,
        ..
    }) = store.kind(opened)
    else {
        return Err("missing opened signature");
    };
    let wrong = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result,
        unsafe_,
        convention,
    })?;
    assert!(opened_signature(&store, wrong, reference, selection).is_err());
    Ok(())
}

fn assert_actual_runtime_origin(
    facts: &crate::sema::model::Declarations,
    body: &ir::Body,
    bodies: &[ir::Body],
    selection: Select,
    call: ir::ValueId,
) -> Result<(), Box<dyn std::error::Error>> {
    let operands = crate::body::operand_origins::inputs(body, call, selection.types)?;
    let mut origins = crate::body::operand_origins::Origins::new(body);
    origins.prepare(&facts.types, |_| Ok(()))?;
    let logical = Instance::new(&facts.types, selection.types[2])?;
    // Preserve the disproved final-logical-type hypothesis.
    assert_eq!(logical, body.callable_target(logical));
    let origin = origins.origin(&facts.types, body, operands[2])?;
    let runtime = Instance::new(&facts.types, body.values[origin.0 as usize].ty)?;
    let target = body.callable_target(runtime);
    assert_ne!(runtime, logical, "the actual Constrain origin must differ");
    assert_ne!(runtime, target, "the actual source origin must be mapped");
    let reached = bodies
        .iter()
        .filter(|other| other.instance == Some(target))
        .collect::<Vec<_>>();
    assert_eq!(
        reached.len(),
        1,
        "actual callback target is demanded exactly once"
    );
    assert!(
        !bodies.iter().any(|other| other.instance == Some(logical)),
        "the constrained logical G must never invent a demanded body"
    );
    let Some(Kind::FunctionItem {
        item, signature, ..
    }) = facts.types.kind(target.ty())
    else {
        return Err("actual target is not a function item".into());
    };
    assert!(
        matches!(reached[0].owner, ir::Owner::Function { definition, signature: public, execution: Some(_), .. } if definition.0 == item.0 && public == signature)
    );
    let source = facts
        .functions
        .iter()
        .find(|function| function.definition.0 == item.0)
        .ok_or("actual source callback declaration")?;
    assert_eq!(reached[0].source.source(), source.body);
    let constant_origin = origins.origin(&facts.types, body, operands[1])?;
    let constant = Instance::new(&facts.types, body.values[constant_origin.0 as usize].ty)?;
    assert!(
        !bodies
            .iter()
            .any(|other| other.instance == Some(body.callable_target(constant))),
        "the unused actual constant callback must not be demanded"
    );
    println!(
        "logical G {logical:?} maps itself; actual origin {runtime:?} -> {target:?}; demanded actual target once, no logical G body or unused F target"
    );
    println!(
        "actual callback source {:?} instance {:?} public/internal {:?}",
        reached[0].source,
        reached[0].instance,
        reached[0].owner.signatures()
    );
    Ok(())
}

#[test]
fn checked_selector_retains_raw_call_identity_and_demands_only_mapped_runtime_callback()
-> Result<(), Box<dyn std::error::Error>> {
    let source =
        std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../pa31/tests/spec/const-eval-select-runtime/mapped-runtime-and-unused-const.rs",
        ))?;
    let report = crate::test_source::source_provider(&source, "selector_mapping")?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    crate::lower::instances::inspect_prepared(
        &report,
        &mut facts,
        prepared,
        |facts, proof| -> Result<(), Box<dyn std::error::Error>> {
            let bodies = &proof.program().bodies;
            let mut found = 0;
            for body in bodies {
                for (index, value) in body.values.iter().enumerate() {
                    let ir::Rvalue::Call {
                        signature, callee, ..
                    } = value.kind
                    else {
                        continue;
                    };
                    let ty = body.values[callee.0 as usize].ty;
                    if !matches!(facts.types.kind(ty), Some(Kind::FunctionItem { .. })) {
                        continue;
                    }
                    let raw = Instance::new(&facts.types, ty)?;
                    let Some(selection) = crate::sema::compiler_items::source_select(facts, raw)?
                    else {
                        continue;
                    };
                    opened_signature(&facts.types, signature, value.ty, selection)?;
                    assert_actual_runtime_origin(
                        facts,
                        body,
                        bodies,
                        selection,
                        ir::ValueId(u32::try_from(index)?),
                    )?;
                    found += 1;
                }
            }
            assert_eq!(found, 1);
            Ok(())
        },
    )??;
    Ok(())
}

#[test]
fn ordinary_source_lowering_allocates_no_callable_origin_index()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::body::operand_origins::take_observations;
    println!(
        "Lower={} Scratch={} RuntimeCall={} Origins={}",
        std::mem::size_of::<Lower<'_, '_, '_>>(),
        std::mem::size_of::<crate::lower::function::Scratch>(),
        std::mem::size_of::<RuntimeCall<'_>>(),
        std::mem::size_of::<crate::body::operand_origins::Origins<'_>>()
    );
    for (name, expected_builds) in [
        ("ordinary-static-target", 0),
        ("pointer-two-targets", 0),
        ("item-empty", 2),
        ("scale-mapped-8", 2),
        ("scale-mapped-32", 2),
        ("scale-mapped-128", 2),
        ("scale-pointer-128", 0),
    ] {
        take_observations();
        let source =
            std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
                format!("../pa31/tests/spec/const-eval-select-runtime/{name}.rs"),
            ))?;
        let report = crate::test_source::source_provider(&source, "origin_allocation")?;
        let crate::pipeline::Analysis {
            mut facts,
            prepared,
        } = crate::pipeline::analyze_source(&report)?;
        crate::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            crate::lower::Roots {
                entry: None,
                exports: &[],
            },
            prepared,
        )?;
        let (builds, rows, payload) = take_observations();
        assert_eq!(builds, expected_builds, "{name}");
        if builds == 0 {
            assert_eq!(rows, 0);
            assert_eq!(payload, 0);
        }
        println!(
            "{name}: source_bytes={} completed_index_builds={builds} completed_index_rows={rows} max_index_payload={payload}",
            source.len()
        );
    }
    Ok(())
}
