//! Cache identity is not permission: malformed requests cannot reuse checked bodies.
use super::{Context, InlineInstance};
use crate::{
    pipeline,
    sema::{
        model::Scalar,
        values::{Constant, execution::Target},
    },
    test_source,
    types::{Primitive, TypeId, instance::Instance},
};
#[test]
fn exact_anonymous_contexts_reject_foreign_owners_and_wrong_results()
-> Result<(), Box<dyn std::error::Error>> {
    let report = test_source::resolve(
        "const fn f<const N:u8>()->u8{const{N}}const A:u8=f::<2>();const B:u8=f::<3>();const fn g<const N:u8>()->u8{const{N}}const C:u8=g::<2>();",
        true,
    )?;
    let pipeline::Analysis {
        mut facts,
        prepared: _,
    } = pipeline::analyze_source(&report)?;
    let values = facts
        .inline_constants
        .as_deref()
        .ok_or("missing registry")?
        .instance_values()
        .collect::<Vec<_>>();
    assert_eq!(values.len(), 3);
    let (instance, result, value) = values[0];
    let other = values
        .iter()
        .find(|(other, _, _)| other.source() != instance.source())
        .ok_or("missing second owner")?
        .0;
    assert!(InlineInstance::select(&facts, instance.source(), other.enclosing()).is_err());
    let wrong = TypeId::primitive(Primitive::U16);
    let mut executor = facts.constant_executor.take().ok_or("missing service")?;
    assert!(
        executor
            .check(
                &report,
                &mut facts,
                Target::Anonymous {
                    instance,
                    result: wrong
                }
            )
            .is_err()
    );
    facts.constant_executor = Some(executor);
    let registry = facts.inline_constants.as_mut().ok_or("lost registry")?;
    assert!(registry.instance_value(instance, result) == Some(value));
    assert!(registry.instance_value(instance, wrong).is_none());
    let publications = registry.instance_publications;
    assert!(
        registry
            .publish_instance(instance, result, Constant::scalar(Scalar::Integer(99)))
            .is_err()
    );
    registry.retained_bytes = 64 * 1024 * 1024;
    assert!(registry.publish_instance(instance, wrong, value).is_err());
    assert_eq!(registry.instance_publications, publications);
    Ok(())
}
#[test]
fn a_matching_owner_with_missing_arguments_is_not_execution_permission()
-> Result<(), Box<dyn std::error::Error>> {
    let report = test_source::resolve(
        "const fn f<const N:u8>()->u8{const{N}}const A:u8=f::<2>();",
        true,
    )?;
    let pipeline::Analysis {
        mut facts,
        prepared: _,
    } = pipeline::analyze_source(&report)?;
    let row = facts
        .inline_constants
        .as_deref()
        .ok_or("missing registry")?
        .bodies()
        .next()
        .ok_or("missing source")?;
    let source = row.source;
    let owner = row.declaration.ok_or("missing owner")?;
    let ty = facts.definitions[owner.0 as usize].ok_or("missing type")?;
    let instance = InlineInstance::select(
        &facts,
        source,
        Context::Callable(Instance::new(&facts.types, ty)?),
    )?;
    let mut executor = facts.constant_executor.take().ok_or("missing service")?;
    assert!(
        executor
            .check(
                &report,
                &mut facts,
                Target::Anonymous {
                    instance,
                    result: TypeId::primitive(Primitive::U8)
                }
            )
            .is_err()
    );
    facts.constant_executor = Some(executor);
    assert_eq!(facts.constant_work.anonymous_instances, 1);
    Ok(())
}

#[test]
fn distinct_closures_in_one_declaration_cannot_exchange_inline_templates()
-> Result<(), Box<dyn std::error::Error>> {
    let mut report = test_source::resolve(
        "fn f<const N:u8>()->u8{let a=||const{N};let b=||const{N+1};a()+b()}fn main(){let _=f::<2>();}",
        true,
    )?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let entry = crate::sema::entry::select(&mut report, &facts)?;
    crate::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
    let values = facts
        .inline_constants
        .as_deref()
        .ok_or("missing registry")?
        .instance_values()
        .collect::<Vec<_>>();
    assert_eq!(values.len(), 2);
    let first = values[0].0;
    let second = values[1].0;
    assert_ne!(first.enclosing(), second.enclosing());
    assert!(InlineInstance::select(&facts, first.source(), second.enclosing()).is_err());
    assert!(InlineInstance::select(&facts, second.source(), first.enclosing()).is_err());
    Ok(())
}

struct Captured {
    report: crate::resolve::expansion::Report,
    facts: crate::sema::model::Declarations,
    values: Vec<(InlineInstance, TypeId, Constant)>,
}
fn captured() -> Result<Captured, Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa25/tests/spec/inline-specialized/captured-erased.rs"),
    )?;
    let mut report = test_source::resolve(&format!("{text}fn main(){{let _=run();}}"), true)?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let entry = crate::sema::entry::select(&mut report, &facts)?;
    crate::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
    let values = facts
        .inline_constants
        .as_deref()
        .ok_or("registry")?
        .instance_values()
        .collect();
    Ok(Captured {
        report,
        facts,
        values,
    })
}
#[test]
fn captured_unused_arguments_retain_complete_value_contexts()
-> Result<(), Box<dyn std::error::Error>> {
    let Captured { facts, values, .. } = captured()?;
    assert_eq!(values.len(), 3);
    assert_eq!(facts.constant_work.anonymous_instances, 3);
    let mut arguments = crate::support::hash::HashSet::default();
    for (instance, _, _) in &values {
        let Context::Callable(callable) = instance.enclosing() else {
            return Err("callable".into());
        };
        let Some(crate::types::Kind::Closure {
            arguments: args,
            environment,
            ..
        }) = facts.types.kind(callable.ty())
        else {
            return Err("closure".into());
        };
        assert_ne!(environment, TypeId::UNIT);
        assert_eq!(facts.types.arguments(args).ok_or("arguments")?.len(), 3);
        arguments.insert(args);
    }
    assert_eq!(arguments.len(), 3);
    assert_eq!(
        values
            .iter()
            .filter(|(_, _, value)| *value == Constant::scalar(Scalar::Integer(2)))
            .count(),
        2
    );
    Ok(())
}
#[test]
fn captured_context_identity_cannot_replace_public_signature_or_erased_slots()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::types::{GenericArgs, Kind};
    let Captured {
        report,
        mut facts,
        values,
    } = captured()?;
    let (inline, result, _) = values[0];
    let Context::Callable(callable) = inline.enclosing() else {
        return Err("callable".into());
    };
    let Some(Kind::Closure {
        identity,
        signature,
        environment,
        arguments,
    }) = facts.types.kind(callable.ty())
    else {
        return Err("closure".into());
    };
    let Some(Kind::Function {
        parameters,
        unsafe_,
        convention,
        ..
    }) = facts.types.kind(signature)
    else {
        return Err("signature".into());
    };
    let wrong_result = facts.types.intern(Kind::Function {
        parameters,
        result: TypeId::primitive(Primitive::U16),
        bound_regions: 0,
        unsafe_,
        convention,
    })?;
    let wrong_binder = facts.types.intern(Kind::Function {
        parameters,
        result,
        bound_regions: 1,
        unsafe_,
        convention,
    })?;
    let args = facts
        .types
        .arguments(arguments)
        .ok_or("arguments")?
        .collect::<Vec<_>>();
    let missing_erased = facts.types.intern_arguments(&args[..2])?;
    let mut executor = facts.constant_executor.take().ok_or("provider")?;
    for (signature, arguments) in [
        (wrong_result, arguments),
        (wrong_binder, arguments),
        (signature, missing_erased),
        (signature, GenericArgs::EMPTY),
    ] {
        let ty = facts.types.intern(Kind::Closure {
            identity,
            signature,
            environment,
            arguments,
        })?;
        let instance = InlineInstance::select(
            &facts,
            inline.source(),
            Context::Callable(Instance::closure(&facts.types, ty)?),
        )?;
        assert!(
            executor
                .check(&report, &mut facts, Target::Anonymous { instance, result })
                .is_err()
        );
        assert!(
            facts
                .inline_constants
                .as_deref()
                .ok_or("registry")?
                .instance_value(instance, result)
                .is_none()
        );
    }
    facts.constant_executor = Some(executor);
    assert_eq!(
        facts
            .inline_constants
            .as_deref()
            .ok_or("registry")?
            .instance_values()
            .count(),
        3
    );
    Ok(())
}

#[test]
fn source_inline_contexts_revalidate_warm_values_and_foreign_catalogs()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::{
        sema::values::execution::{Progress, Request},
        types::arguments::Argument,
    };
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa25/tests/spec/inline-query-context/required.rs");
    let text = std::fs::read_to_string(path)?;
    let mut report = test_source::resolve(&text, true)?;
    let analysis = pipeline::analyze_source(&report)?;
    let mut facts = analysis.facts;
    let entry = crate::sema::entry::select(&mut report, &facts)?;
    crate::lower::instances::build_prepared(&report, &mut facts, entry, analysis.prepared)?;
    let (instance, result, _) = facts
        .inline_constants
        .as_deref()
        .ok_or("missing inline registry")?
        .instance_values()
        .find(|(instance, _, _)| matches!(instance.enclosing(), Context::SourceInitializer(_)))
        .ok_or("missing source inline value")?;
    let Context::SourceInitializer(witness) = instance.enclosing() else {
        return Err("source context".into());
    };
    let selected = witness.identity(&facts)?.0;
    let request = Request {
        target: Target::Anonymous { instance, result },
        arguments: Vec::new(),
    };
    let mut executor = facts.constant_executor.take().ok_or("missing executor")?;
    assert!(matches!(
        executor.advance(&report, &mut facts, &request)?,
        Progress::Complete(_)
    ));
    let region = facts
        .types
        .arguments(selected.arguments)
        .and_then(|mut args| {
            args.find_map(|arg| match arg {
                Argument::Lifetime(r) if r != crate::types::RegionId::STATIC => Some(r),
                _ => None,
            })
        })
        .ok_or("missing caller lifetime")?;
    let old = facts.body_regions[region.0 as usize - 1];
    facts.body_regions[region.0 as usize - 1].body.node = crate::syntax::ast::NodeId::new(
        facts.body_regions[region.0 as usize - 1].body.node.get() + 1,
    );
    assert!(executor.check(&report, &mut facts, request.target).is_err());
    assert!(executor.advance(&report, &mut facts, &request).is_err());
    facts.body_regions[region.0 as usize - 1] = old;
    assert!(executor.check(&report, &mut facts, request.target).is_ok());
    let row = facts
        .inline_constants
        .as_deref_mut()
        .ok_or("inline registry")?
        .bodies
        .iter_mut()
        .find(|body| body.source == instance.source())
        .ok_or("inline body")?;
    let old_result = row.result.take();
    assert!(InlineInstance::select(&facts, instance.source(), instance.enclosing()).is_err());
    assert!(executor.check(&report, &mut facts, request.target).is_err());
    facts
        .inline_constants
        .as_deref_mut()
        .ok_or("inline registry")?
        .bodies
        .iter_mut()
        .find(|body| body.source == instance.source())
        .ok_or("inline body")?
        .result = old_result;
    assert!(
        executor
            .check(
                &report,
                &mut facts,
                Target::Anonymous {
                    instance,
                    result: TypeId::UNIT
                }
            )
            .is_err()
    );
    facts.constant_executor = Some(executor);
    let foreign_report = test_source::resolve(&text, true)?;
    let foreign = pipeline::analyze_source(&foreign_report)?.facts;
    assert!(InlineInstance::select(&foreign, instance.source(), instance.enclosing()).is_err());
    println!(
        "source inline layout Context={} InlineInstance={} BodyKey={} Target={} Request={}",
        size_of::<Context>(),
        size_of::<InlineInstance>(),
        size_of::<crate::body::ir::BodyKey>(),
        size_of::<Target>(),
        size_of::<Request>()
    );
    Ok(())
}
