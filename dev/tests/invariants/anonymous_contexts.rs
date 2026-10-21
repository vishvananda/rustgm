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
