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
    assert_eq!(completed.ty, TypeId::primitive(Primitive::U8));
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
                    target: Target::Associated {
                        instance,
                        result: completed.ty
                    },
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
                    result: completed.ty
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
                Some(completed.ty),
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
            ty: completed.ty,
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
            .publish(&mut facts, Some(completed.ty), completed.value)
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
                    result: completed.ty,
                }
            )
            .is_err()
    );
    assert!(
        malformed
            .publish(&mut facts, Some(completed.ty), completed.value)
            .is_err()
    );
    assert!(Reader::new(&facts).associated(malformed).is_none());
    assert_eq!(facts.constant_instances.len(), 1);
    facts.constant_executor = Some(executor);
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
