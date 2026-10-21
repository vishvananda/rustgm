//! Receiver wrappers require their exact source proof and retain ordinary lifetime debts.
use super::{Fixture, TestResult, fixture, originals};
use crate::{
    body::{
        self,
        ir::Owner,
        regions::{Contract, Kind, Obligation},
    },
    borrow::{self, templates},
    sema::model::Declarations,
    types::{self, RegionId, instance::Instance},
};
const SOURCE: &str =
    "fn call<F:FnOnce()->u8>(f:F)->u8{f()}fn main(){let n=7u8;let f=||n;let _=call(f);}";
fn generic_entry(
    facts: &mut Declarations,
    checked: &templates::Checked<'_>,
    outer: Instance,
) -> TestResult<body::invocation_entries::Entry> {
    let Some(types::Kind::FunctionItem { item, .. }) = facts.types.kind(outer.ty()) else {
        return Err("outer identity".into());
    };
    let mut cleanup = body::instantiate::Cleanup::new(facts);
    let mut outer = body::instantiate::from_checked_caller(
        facts,
        checked.get(item).ok_or("outer template")?,
        outer,
        checked.input(item).ok_or("outer contract")?,
        &mut cleanup,
    )?;
    body::dispatch::resolve(facts, &mut outer.body, &mut outer.contract)?;
    let caller = outer
        .body
        .values
        .iter()
        .find_map(|value| {
            let types::Kind::FunctionItem { item, .. } = facts.types.kind(value.ty)? else {
                return None;
            };
            checked
                .get(item)?
                .body
                .values
                .iter()
                .any(|v| matches!(v.kind, body::ir::Rvalue::Invoke { .. }))
                .then_some(value.ty)
        })
        .ok_or("selected nested caller")?;
    selected(facts, checked, Instance::new(&facts.types, caller)?)
}

#[test]
fn generic_adapter_requires_its_exact_specialized_source() -> TestResult {
    let source = "fn once<F:FnOnce()->u8>(f:F)->u8{f()}fn outer<T:Copy>(x:u8)->u8{let f=move||x;once(f)}fn main(){let _=outer::<u8>(7);let _=outer::<u16>(9);}";
    let Fixture {
        mut facts,
        templates,
        inputs,
        instances,
        program,
    } = fixture(source)?;
    let checked = originals(&mut facts, &templates, &inputs)?;
    assert_eq!(instances.len(), 2);
    let error = generic_entry(&mut facts, &checked, instances[0])
        .err()
        .ok_or("missing source conditions authorized an adapter")?;
    assert!(
        error
            .to_string()
            .contains("checked source execution conditions")
    );
    checked.visit_closure_conditions(|identity, source, conditions| {
        facts
            .closures
            .retain_conditions(identity, source, conditions)
    })?;
    let mut entries = Vec::new();
    for outer in instances {
        entries.push(generic_entry(&mut facts, &checked, outer)?);
    }
    assert_ne!(entries[0].instance(), entries[1].instance());
    let (mut builder, _) = super::super::Builder::new(program)?;
    let mut indices = Vec::new();
    for entry in &entries {
        let Some(types::Kind::Closure { identity, .. }) = facts.types.kind(entry.instance().ty())
        else {
            return Err("closure instance".into());
        };
        let (template, input) = checked.closure(identity).ok_or("checked source template")?;
        let mut cleanup = body::instantiate::Cleanup::new(&facts);
        let mut original = body::instantiate::from_checked_caller(
            &mut facts,
            template,
            entry.instance(),
            input,
            &mut cleanup,
        )?;
        body::dispatch::resolve(&mut facts, &mut original.body, &mut original.contract)?;
        let mut forged = original.body.clone();
        if let Owner::Closure { execution, .. } = &mut forged.owner {
            *execution = Some(types::TypeId::UNIT);
        }
        assert!(
            builder
                .push(&mut facts, &checked, forged, &original.contract, None)
                .is_err()
        );
        indices.push(builder.push(
            &mut facts,
            &checked,
            original.body,
            &original.contract,
            Some(original.destruction),
        )?);
    }
    let report = crate::test_source::resolve(source, true)?;
    assert!(body::adapters::build(&mut facts, entries[0], builder.body(indices[1])?).is_err());
    let prepared = body::adapters::build(&mut facts, entries[0], builder.body(indices[0])?)?;
    assert!(
        builder
            .push_adapter(&report, &mut facts, &checked, indices[1], prepared)
            .is_err()
    );
    let prepared = body::adapters::build(&mut facts, entries[0], builder.body(indices[0])?)?;
    let mut unpaid = prepared;
    unpaid.input.obligations.push(Obligation {
        source: unpaid.body.source,
        kind: Kind::Outlives((RegionId(1), RegionId::STATIC)),
    });
    assert!(
        builder
            .push_adapter(&report, &mut facts, &checked, indices[0], unpaid)
            .is_err()
    );
    let prepared = body::adapters::build(&mut facts, entries[0], builder.body(indices[0])?)?;
    builder.push_adapter(&report, &mut facts, &checked, indices[0], prepared)?;
    Ok(())
}
fn selected(
    facts: &mut Declarations,
    checked: &templates::Checked<'_>,
    instance: Instance,
) -> TestResult<body::invocation_entries::Entry> {
    let Some(types::Kind::FunctionItem { item, .. }) = facts.types.kind(instance.ty()) else {
        return Err("missing selected caller".into());
    };
    let mut cleanup = body::instantiate::Cleanup::new(facts);
    let mut materialized = body::instantiate::from_checked_caller(
        facts,
        checked.get(item).ok_or("caller template")?,
        instance,
        checked.input(item).ok_or("caller contract")?,
        &mut cleanup,
    )?;
    body::dispatch::resolve(facts, &mut materialized.body, &mut materialized.contract)?;
    materialized
        .body
        .values
        .iter()
        .enumerate()
        .find_map(|(i, _)| {
            let id = body::ir::ValueId(u32::try_from(i).ok()?);
            materialized
                .body
                .closure_calls
                .as_ref()?
                .get(id)
                .filter(|entry| entry.adapter().is_some())
        })
        .ok_or_else(|| "selected caller has no adapter".into())
}
fn proved_sources(
    facts: &mut Declarations,
    checked: &templates::Checked<'_>,
    program: body::ir::Program,
    inputs: &[Contract],
) -> TestResult<(super::super::Builder, usize, usize)> {
    let (mut builder, bodies) = super::super::Builder::new(program)?;
    let inputs: std::collections::HashMap<_, _> =
        inputs.iter().cloned().map(|c| (c.key, c)).collect();
    builder.prepare_original_regions(facts, &inputs, &bodies)?;
    let mut source = None;
    let mut wrong = None;
    for body in bodies {
        let closure = matches!(body.owner, Owner::Closure { .. });
        let input = inputs.get(&body.key()).ok_or("original input")?;
        let index = builder.push(facts, checked, body, input, None)?;
        if closure {
            source = Some(index);
        } else {
            wrong = Some(index);
        }
    }
    Ok((
        builder,
        source.ok_or("source proof")?,
        wrong.ok_or("opposing source proof")?,
    ))
}
#[test]
fn receiver_adapter_requires_the_exact_sealed_source() -> TestResult {
    let Fixture {
        mut facts,
        templates,
        inputs,
        instances,
        program,
    } = fixture(SOURCE)?;
    let checked = originals(&mut facts, &templates, &inputs)?;
    let entry = selected(&mut facts, &checked, *instances.first().ok_or("instance")?)?;
    let (mut builder, source, wrong) = proved_sources(&mut facts, &checked, program, &inputs)?;
    let report = crate::test_source::resolve(SOURCE, true)?;
    let prepared = body::adapters::build(&mut facts, entry, builder.body(source)?)?;
    let error = builder
        .push(&mut facts, &checked, prepared.body, &prepared.input, None)
        .err()
        .ok_or("ordinary proof authorized an adapter")?;
    assert!(error.message.contains("checked original source"), "{error}");
    let prepared = body::adapters::build(&mut facts, entry, builder.body(source)?)?;
    let error = builder
        .push_adapter(&report, &mut facts, &checked, wrong, prepared)
        .err()
        .ok_or("wrong source index authorized an adapter")?;
    assert!(error.message.contains("different proved source"), "{error}");
    let mut prepared = body::adapters::build(&mut facts, entry, builder.body(source)?)?;
    let Owner::ClosureAdapter { adapter, .. } = prepared.body.owner else {
        return Err("adapter owner".into());
    };
    prepared.body.owner = Owner::ClosureAdapter {
        adapter,
        signature: types::TypeId::UNIT,
    };
    let error = builder
        .push_adapter(&report, &mut facts, &checked, source, prepared)
        .err()
        .ok_or("forged signature authorized an adapter")?;
    assert!(error.message.contains("contract differs"), "{error}");
    let mut prepared = body::adapters::build(&mut facts, entry, builder.body(source)?)?;
    let forged = types::closure::Adapter::new(
        &facts.types,
        entry.instance(),
        crate::support::call_traits::CallTrait::FnMut,
    )?;
    let Owner::ClosureAdapter { signature, .. } = prepared.body.owner else {
        return Err("adapter owner".into());
    };
    prepared.body.owner = Owner::ClosureAdapter {
        adapter: forged,
        signature,
    };
    prepared.input.key = body::ir::BodyKey::ClosureAdapter(forged);
    let error = builder
        .push_adapter(&report, &mut facts, &checked, source, prepared)
        .err()
        .ok_or("forged matching owner/key authorized an adapter")?;
    assert!(
        error.message.contains("different materialized wrapper"),
        "{error}"
    );
    let prepared = body::adapters::build(&mut facts, entry, builder.body(source)?)?;
    let index = builder.push_adapter(&report, &mut facts, &checked, source, prepared)?;
    assert!(matches!(
        builder.body(index)?.owner,
        Owner::ClosureAdapter { .. }
    ));
    assert!(matches!(builder.body(source)?.owner, Owner::Closure { .. }));
    Ok(())
}
#[test]
fn private_receiver_borrow_cannot_pay_a_static_lifetime() -> TestResult {
    let Fixture {
        mut facts,
        templates,
        inputs,
        instances,
        program,
    } = fixture(SOURCE)?;
    let checked = originals(&mut facts, &templates, &inputs)?;
    let entry = selected(&mut facts, &checked, *instances.first().ok_or("instance")?)?;
    let (mut builder, source, _) = proved_sources(&mut facts, &checked, program, &inputs)?;
    let report = crate::test_source::resolve(SOURCE, true)?;
    let mut prepared = body::adapters::build(&mut facts, entry, builder.body(source)?)?;
    prepared.input.obligations.push(Obligation {
        source: prepared.body.source,
        kind: Kind::Outlives((RegionId(1), RegionId::STATIC)),
    });
    assert!(
        builder
            .push_adapter(&report, &mut facts, &checked, source, prepared)
            .is_err()
    );
    Ok(())
}
#[test]
fn source_instance_validation_checks_the_complete_environment() -> TestResult {
    let Fixture {
        mut facts,
        templates,
        inputs,
        instances,
        program,
    } = fixture(SOURCE)?;
    let checked = originals(&mut facts, &templates, &inputs)?;
    let entry = selected(&mut facts, &checked, *instances.first().ok_or("instance")?)?;
    let (builder, source, _) = proved_sources(&mut facts, &checked, program, &inputs)?;
    let Owner::Closure {
        execution: Some(execution),
        ..
    } = builder.body(source)?.owner
    else {
        return Err("source execution".into());
    };
    let Some(types::Kind::Closure {
        identity,
        signature,
        arguments,
        ..
    }) = facts.types.kind(entry.instance().ty())
    else {
        return Err("closure instance".into());
    };
    let environment = types::TypeId::UNIT;
    let forged = facts.types.intern(types::Kind::Closure {
        identity,
        signature,
        arguments,
        environment,
    })?;
    let forged = Instance::closure(&facts.types, forged)?;
    assert!(body::invocation_entries::validate_instance(&facts.types, execution, forged).is_err());
    Ok(())
}
#[test]
fn external_reference_result_uses_capture_lifetime() -> TestResult {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa24/tests/spec/receiver-adapters/once-keeps-external-reference.rs"),
    )?;
    let mut report = crate::test_source::resolve(&source, true)?;
    let analysis = crate::pipeline::analyze_source(&report)?;
    let mut facts = analysis.facts;
    let entry = crate::sema::entry::select(&mut report, &facts)?;
    crate::lower::instances::build_prepared(&report, &mut facts, entry, analysis.prepared)?;
    Ok(())
}
#[test]
fn only_the_sealed_caller_inherits_its_declared_output_equality() -> TestResult {
    let mut f = fixture(
        "fn call<'a,F:FnOnce()->&'a u8>(f:F)->&'a u8{f()}fn main(){let n=7u8;let f=||&n;let _=call(f);}",
    )?;
    let checked = originals(&mut f.facts, &f.templates, &f.inputs)?;
    let instance = *f.instances.first().ok_or("caller instance")?;
    let Some(types::Kind::FunctionItem { item, .. }) = f.facts.types.kind(instance.ty()) else {
        return Err("caller identity".into());
    };
    let template = checked.get(item).ok_or("template")?;
    let input = checked.input(item).ok_or("contract")?;
    let manual = body::instantiate::with_contract(&mut f.facts, template, instance, input)?;
    let mut cleanup = body::instantiate::Cleanup::new(&f.facts);
    let declared = body::instantiate::from_checked_caller(
        &mut f.facts,
        template,
        instance,
        input,
        &mut cleanup,
    )?;
    assert!(manual.contract.assumptions.is_empty());
    assert!(!declared.contract.assumptions.is_empty());
    for (mut value, expected) in [(manual, false), (declared, true)] {
        body::dispatch::resolve(&mut f.facts, &mut value.body, &mut value.contract)?;
        let mut queries = types::outlives::Queries::new(
            &f.facts.types,
            &f.facts.lifetimes.nominal_requirements,
            &f.facts.lifetimes.callables,
        );
        assert_eq!(
            borrow::regions::check_one(
                &f.facts.types,
                &f.facts.lifetimes.variances,
                &mut queries,
                &value.contract,
                &value.body
            )
            .is_ok(),
            expected
        );
    }
    Ok(())
}
#[test]
fn distinct_callers_share_only_the_two_exact_receiver_entries() -> TestResult {
    use std::fmt::Write as _;
    for width in [8, 32, 128] {
        let mut source = String::from("fn id<T>(x:T)->T{x}");
        for index in 0..width {
            let (bound, binding) = if index % 2 == 0 {
                ("FnMut", "mut f")
            } else {
                ("FnOnce", "f")
            };
            write!(
                source,
                "fn call{index}<F:{bound}()->u8>({binding}:F)->u8{{f()}}"
            )?;
        }
        source.push_str("fn main(){let n=7u8;let f=||n;");
        for index in 0..width {
            write!(source, "let _=id(call{index}(f));")?;
        }
        source.push('}');
        let mut report = crate::test_source::resolve(&source, true)?;
        let analysis = crate::pipeline::analyze_source(&report)?;
        let mut facts = analysis.facts;
        let entry = crate::sema::entry::select(&mut report, &facts)?;
        crate::lower::instances::inspect_roots(
            &report,
            &mut facts,
            crate::lower::Roots::executable(entry),
            analysis.prepared,
            |facts, checked, _| -> TestResult {
                let bodies = &checked.program().bodies;
                let adapters: Vec<_> = bodies
                    .iter()
                    .filter_map(|b| {
                        if let Owner::ClosureAdapter { adapter, .. } = b.owner {
                            Some((b, adapter))
                        } else {
                            None
                        }
                    })
                    .collect();
                assert_eq!(adapters.len(), 2);
                assert_eq!(adapters[0].1.instance(), adapters[1].1.instance());
                assert_ne!(adapters[0].1.requested(), adapters[1].1.requested());
                for (wrapper, adapter) in &adapters {
                    let Some(types::Kind::Closure { identity, .. }) =
                        facts.types.kind(adapter.instance().ty())
                    else {
                        return Err("source closure".into());
                    };
                    assert!(bodies.iter().any(|b| matches!(b.owner, Owner::Closure { identity: actual, .. } if identity == actual)));
                    assert!(wrapper.values.len() <= 3);
                }
                let bytes: usize = bodies.iter().map(body::ir::Body::retained_bytes).sum();
                println!(
                    "receiver adapters width={width} bodies={} wrappers={} bytes={bytes}",
                    bodies.len(),
                    adapters.len()
                );
                assert!(bodies.len() <= width + 8);
                assert!(bytes <= 20_000 * width + 20_000);
                Ok(())
            },
        )??;
    }
    Ok(())
}
#[test]
fn equal_projection_results_do_not_equate_their_inputs() -> TestResult {
    let mut f = fixture(
        "fn mix<F:FnOnce()->u8,G:FnOnce()->u8>(f:F,g:G)->u8{let _=f();g()}fn main(){let n=7u8;let m=9u8;let f=||n;let g=||m;let _=mix(f,g);}",
    )?;
    let checked = originals(&mut f.facts, &f.templates, &f.inputs)?;
    let instance = *f.instances.first().ok_or("instance")?;
    let Some(types::Kind::FunctionItem { item, .. }) = f.facts.types.kind(instance.ty()) else {
        return Err("function item".into());
    };
    let mut cleanup = body::instantiate::Cleanup::new(&f.facts);
    let value = body::instantiate::from_checked_caller(
        &mut f.facts,
        checked.get(item).ok_or("template")?,
        instance,
        checked.input(item).ok_or("input")?,
        &mut cleanup,
    )?;
    assert!(value.contract.universals >= 2);
    assert!(value.contract.assumptions.is_empty());
    Ok(())
}
#[test]
fn borrowed_self_is_part_of_the_key_and_of_the_checked_execution() -> TestResult {
    let source_text =
        "fn call<F:FnOnce()->u8>(f:F)->u8{f()}fn main(){let n=7u8;let f=||n;let _=call(&f);}";
    let Fixture {
        mut facts,
        templates,
        inputs,
        instances,
        program,
    } = fixture(source_text)?;
    let checked = originals(&mut facts, &templates, &inputs)?;
    let entry = selected(&mut facts, &checked, *instances.first().ok_or("instance")?)?;
    let adapter = entry.adapter().ok_or("adapter")?;
    assert_ne!(adapter.receiver(), adapter.instance().ty());
    let owned =
        types::closure::Adapter::new(&facts.types, adapter.instance(), adapter.requested())?;
    assert_ne!(owned, adapter);
    let (mut builder, source, _) = proved_sources(&mut facts, &checked, program, &inputs)?;
    let report = crate::test_source::resolve(source_text, true)?;
    let mut prepared = body::adapters::build(&mut facts, entry, builder.body(source)?)?;
    let Owner::ClosureAdapter { signature, .. } = prepared.body.owner else {
        return Err("owner".into());
    };
    prepared.body.owner = Owner::ClosureAdapter {
        adapter: owned,
        signature,
    };
    prepared.input.key = body::ir::BodyKey::ClosureAdapter(owned);
    let error = builder
        .push_adapter(&report, &mut facts, &checked, source, prepared)
        .err()
        .ok_or("owned key authorized borrowed Self ABI")?;
    assert!(
        error.message.contains("different materialized wrapper"),
        "{error}"
    );
    Ok(())
}
#[test]
fn distinct_borrowed_callers_cache_exact_self_entries() -> TestResult {
    use std::fmt::Write as _;
    for width in [8, 32, 128] {
        let mut source = String::from("fn id<T>(x:T)->T{x}");
        for index in 0..width {
            let (bound, binding) = if index % 2 == 0 {
                ("FnMut", "mut f")
            } else {
                ("FnOnce", "f")
            };
            write!(
                source,
                "fn call{index}<F:{bound}()->u8>({binding}:F)->u8{{f()}}"
            )?;
        }
        source.push_str("fn main(){let n=7u8;let f=||n;");
        for index in 0..width {
            write!(source, "let _=id(call{index}(&f));")?;
        }
        source.push('}');
        let mut report = crate::test_source::resolve(&source, true)?;
        let analysis = crate::pipeline::analyze_source(&report)?;
        let mut facts = analysis.facts;
        let entry = crate::sema::entry::select(&mut report, &facts)?;
        crate::lower::instances::inspect_roots(
            &report,
            &mut facts,
            crate::lower::Roots::executable(entry),
            analysis.prepared,
            |facts, checked, _| -> TestResult {
                let bodies = &checked.program().bodies;
                let adapters: Vec<_> = bodies
                    .iter()
                    .filter_map(|b| {
                        if let Owner::ClosureAdapter { adapter, .. } = b.owner {
                            Some((b, adapter))
                        } else {
                            None
                        }
                    })
                    .collect();
                assert_eq!(adapters.len(), 2);
                assert_eq!(adapters[0].1.receiver(), adapters[1].1.receiver());
                for (wrapper, adapter) in &adapters {
                    assert!(matches!(
                        facts.types.kind(adapter.receiver()),
                        Some(types::Kind::Reference { .. })
                    ));
                    assert_ne!(adapter.receiver(), adapter.instance().ty());
                    assert_eq!(
                        wrapper.projections.len(),
                        if adapter.requested() == crate::support::call_traits::CallTrait::FnOnce {
                            1
                        } else {
                            2
                        }
                    );
                }
                let bytes: usize = bodies.iter().map(body::ir::Body::retained_bytes).sum();
                println!(
                    "borrowed adapters width={width} bodies={} wrappers={} bytes={bytes}",
                    bodies.len(),
                    adapters.len()
                );
                assert!(bodies.len() <= width + 8);
                assert!(bytes <= 24_000 * width + 24_000);
                Ok(())
            },
        )??;
    }
    Ok(())
}
