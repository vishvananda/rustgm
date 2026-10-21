//! Completed selected values cannot transfer type or initializer authority.
use crate::{
    pipeline,
    sema::model::Scalar,
    sema::values::{
        Constant, Reader,
        execution::{Request, Target},
    },
    test_source,
    types::{GenericArgs, Primitive, TypeId},
};
#[test]
fn selected_cache_checks_canonical_results_before_reuse() -> Result<(), Box<dyn std::error::Error>>
{
    let (report, mut facts) = facts()?;
    let values = facts
        .constant_instances
        .iter()
        .map(|(&instance, completed)| (instance, *completed))
        .collect::<Vec<_>>();
    assert_eq!(values.len(), 2);
    let (instance, completed) = values[0];
    let result = completed.ty.ok_or("missing materialized result")?;
    assert_eq!(result, TypeId::primitive(Primitive::U8));
    assert!(
        Reader::new(&facts)
            .definition(instance.definition)
            .is_none()
    );
    let wrong = TypeId::primitive(Primitive::U16);
    let mut executor = facts.constant_executor.take().ok_or("missing provider")?;
    assert!(
        executor
            .check(
                &report,
                &mut facts,
                Target::Associated {
                    instance,
                    result: wrong
                }
            )
            .is_err()
    );
    assert!(
        executor
            .advance(
                &report,
                &mut facts,
                &Request {
                    target: Target::Associated {
                        instance,
                        result: wrong
                    },
                    arguments: Vec::new()
                }
            )
            .is_err()
    );
    assert!(
        executor
            .advance(
                &report,
                &mut facts,
                &Request {
                    target: Target::Associated { instance, result },
                    arguments: vec![Constant::scalar(Scalar::Integer(7))]
                }
            )
            .is_err()
    );
    let malformed = super::Instance {
        definition: instance.definition,
        arguments: GenericArgs::EMPTY,
    };
    assert!(
        executor
            .check(
                &report,
                &mut facts,
                Target::Associated {
                    instance: malformed,
                    result
                }
            )
            .is_err()
    );
    facts.constant_executor = Some(executor);
    assert!(Reader::new(&facts).associated(instance) == Some(completed.value));
    assert_eq!(facts.constant_instances.len(), 2);
    Ok(())
}
#[test]
fn selected_publication_preserves_immutability_and_storage_bounds()
-> Result<(), Box<dyn std::error::Error>> {
    let (_, mut facts) = facts()?;
    let (instance, completed) = facts
        .constant_instances
        .iter()
        .next()
        .map(|(&instance, &value)| (instance, value))
        .ok_or("missing completed value")?;
    let result = completed.ty.ok_or("missing materialized result")?;
    let wrong = TypeId::primitive(Primitive::U16);
    assert!(
        instance
            .publish(&mut facts, Some(wrong), completed.value)
            .is_err()
    );
    assert!(
        instance
            .publish(
                &mut facts,
                Some(result),
                Constant::scalar(Scalar::Integer(99))
            )
            .is_err()
    );
    assert!(Reader::new(&facts).associated(instance) == Some(completed.value));
    assert_eq!(facts.constant_instances.len(), 2);
    let mut arguments = facts
        .types
        .arguments(instance.arguments)
        .ok_or("missing arguments")?
        .collect::<Vec<_>>();
    let constant = facts
        .types
        .intern_constant(crate::types::arguments::Constant {
            ty: result,
            bits: 99,
        })?;
    let mut changed = false;
    for argument in &mut arguments {
        if matches!(argument, crate::types::arguments::Argument::Const(_)) {
            *argument = crate::types::arguments::Argument::Const(constant);
            changed = true;
        }
    }
    assert!(changed);
    let fresh = super::Instance {
        definition: instance.definition,
        arguments: facts.types.intern_arguments(&arguments)?,
    };
    assert!(Reader::new(&facts).associated(fresh).is_none());
    facts.constant_work.associated_value_bytes = 64 * 1024 * 1024;
    assert!(
        fresh
            .publish(&mut facts, Some(result), completed.value)
            .is_err()
    );
    assert!(Reader::new(&facts).associated(fresh).is_none());
    assert_eq!(facts.constant_instances.len(), 2);
    Ok(())
}

#[test]
fn erased_owner_parameters_cannot_supply_selected_execution_identity()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::types::{Kind, arguments::Argument};
    let report = test_source::resolve(
        "struct S<T>(T);impl<T>S<T>{const C:u8={let value=7;value};}const A:u8=S::<u8>::C;",
        true,
    )?;
    let pipeline::Analysis { mut facts, .. } = pipeline::analyze_source(&report)?;
    let (&instance, &completed) = facts
        .constant_instances
        .iter()
        .next()
        .ok_or("missing selected value")?;
    let result = completed.ty.ok_or("missing materialized result")?;
    let (_, parameters) = facts
        .generics
        .callable_parameters(&facts.types, instance.definition)?;
    assert_eq!(parameters.len(), 1);
    let parameter = facts.types.intern(Kind::Parameter(parameters[0]))?;
    let malformed = super::Instance {
        definition: instance.definition,
        arguments: facts.types.intern_arguments(&[Argument::Type(parameter)])?,
    };
    let mut executor = facts.constant_executor.take().ok_or("missing provider")?;
    assert!(
        executor
            .check(
                &report,
                &mut facts,
                Target::Associated {
                    instance: malformed,
                    result,
                }
            )
            .is_err()
    );
    assert!(
        malformed
            .publish(&mut facts, Some(result), completed.value)
            .is_err()
    );
    assert!(Reader::new(&facts).associated(malformed).is_none());
    assert_eq!(facts.constant_instances.len(), 1);
    facts.constant_executor = Some(executor);
    Ok(())
}

#[test]
fn symbolic_source_values_do_not_authorize_initializer_execution()
-> Result<(), Box<dyn std::error::Error>> {
    for path in [
        "selection/lookup-readiness/semantics/trait-caller-lifetime.rs",
        "constants/pattern/semantics/erased-type-called.rs",
    ] {
        let source = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../pa20/tests/spec")
                .join(path),
        )?;
        let report = test_source::resolve(&source, true)?;
        let pipeline::Analysis { mut facts, .. } = pipeline::analyze_source(&report)?;
        let values = facts
            .constant_instances
            .iter()
            .map(|(&instance, &value)| (instance, value))
            .collect::<Vec<_>>();
        let count = values.len();
        let mut executor = facts.constant_executor.take().ok_or("missing provider")?;
        let mut symbolic = 0;
        for (instance, completed) in values {
            if !facts.types.arguments_are_concrete(instance.arguments)? {
                symbolic += 1;
                assert!(completed.ty.is_none());
                let result = facts.definitions[instance.definition.0 as usize]
                    .ok_or("missing declared type")?;
                assert!(
                    executor
                        .check(&report, &mut facts, Target::Associated { instance, result })
                        .is_err()
                );
                assert!(
                    instance
                        .publish(&mut facts, Some(result), completed.value)
                        .is_err()
                );
                assert!(Reader::new(&facts).associated(instance) == Some(completed.value));
            }
        }
        assert!(symbolic > 0);
        assert_eq!(facts.constant_instances.len(), count);
        facts.constant_executor = Some(executor);
    }
    Ok(())
}

#[test]
fn source_publication_upgrades_only_an_identical_checked_value()
-> Result<(), Box<dyn std::error::Error>> {
    let (_, mut facts) = facts()?;
    let (&instance, &completed) = facts
        .constant_instances
        .iter()
        .next()
        .ok_or("missing value")?;
    let result = completed.ty.ok_or("missing materialized type")?;
    facts.constant_instances.remove(&instance);
    instance.publish(&mut facts, None, completed.value)?;
    assert!(facts.constant_instances[&instance].ty.is_none());
    assert!(
        instance
            .publish(
                &mut facts,
                Some(TypeId::primitive(Primitive::U16)),
                completed.value
            )
            .is_err()
    );
    assert!(
        instance
            .publish(
                &mut facts,
                Some(result),
                Constant::scalar(Scalar::Integer(99))
            )
            .is_err()
    );
    assert!(facts.constant_instances[&instance].ty.is_none());
    instance.publish(&mut facts, Some(result), completed.value)?;
    assert_eq!(facts.constant_instances[&instance].ty, Some(result));
    instance.publish(&mut facts, None, completed.value)?;
    assert_eq!(facts.constant_instances[&instance].ty, Some(result));
    assert!(
        instance
            .publish(&mut facts, None, Constant::scalar(Scalar::Integer(99)))
            .is_err()
    );
    assert!(Reader::new(&facts).associated(instance) == Some(completed.value));
    Ok(())
}

fn facts() -> Result<
    (
        crate::resolve::expansion::Report,
        crate::sema::model::Declarations,
    ),
    Box<dyn std::error::Error>,
> {
    let report = test_source::resolve(
        "struct S<const N:u8>;impl<const N:u8>S<N>{const C:u8={let n=N;n+1};}const A:u8=S::<2>::C;const B:u8=S::<3>::C;",
        true,
    )?;
    let pipeline::Analysis { facts, prepared: _ } = pipeline::analyze_source(&report)?;
    Ok((report, facts))
}
