//! The assumption index must bound unrelated candidates without guessing unknown Self.
use super::{Assumptions, Bound, Context, ItemId, Kind, Predicate, Primitive, Store};
use crate::types::{
    TypeId,
    arguments::Argument,
    infer::{Domain, Instantiation},
    parameters::{Parameter, ParameterKind},
    traits::Application,
};

#[test]
fn unrelated_rigid_subjects_do_not_make_proof_quadratic() -> Result<(), Box<dyn std::error::Error>>
{
    let mut store = Store::default();
    let arguments = store.intern_arguments(&[Argument::Type(TypeId::primitive(Primitive::U8))])?;
    let application = store.intern_trait_application(Application {
        item: ItemId(1),
        arguments,
    })?;
    let mut assumptions = Assumptions::default();
    let mut predicates = Vec::new();
    for index in 0..4096 {
        let parameter = store.intern_parameter(Parameter {
            owner: ItemId(2),
            index,
            kind: ParameterKind::Type { sized: true },
        })?;
        let predicate = Predicate {
            ty: store.intern(Kind::Parameter(parameter))?,
            bound: Bound::Trait(application),
        };
        assumptions.insert(&store, predicate)?;
        assumptions.insert(&store, predicate)?;
        predicates.push(predicate);
    }
    let mut context = Context::default();
    let mut substitution = Instantiation::new(&mut context, &store, &[])?;
    for predicate in predicates {
        assert!(assumptions.unambiguous_declaration(&mut store, &mut context, predicate)?);
        let goal = context.instantiate_predicate(&mut store, predicate, &mut substitution)?;
        let evidence = assumptions
            .prove(&store, &mut context, &goal)?
            .ok_or("missing unique assumption")?;
        assert_eq!(evidence.assumption(), predicate);
    }
    assert!(context.steps < 150_000, "{}", context.steps);
    Ok(())
}

#[test]
fn unknown_self_stays_unresolved_even_with_one_candidate() -> Result<(), Box<dyn std::error::Error>>
{
    let mut store = Store::default();
    let arguments = store.intern_arguments(&[])?;
    let application = store.intern_trait_application(Application {
        item: ItemId(1),
        arguments,
    })?;
    let parameter = store.intern_parameter(Parameter {
        owner: ItemId(2),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let predicate = Predicate {
        ty: store.intern(Kind::Parameter(parameter))?,
        bound: Bound::Trait(application),
    };
    let mut assumptions = Assumptions::default();
    assumptions.insert(&store, predicate)?;
    let mut context = Context::default();
    let unknown = context.variable(Domain::Any)?;
    let mut substitution = Instantiation::new(
        &mut context,
        &store,
        &[(
            parameter,
            crate::types::infer::arguments::Argument::Type(unknown),
        )],
    )?;
    let goal = context.instantiate_predicate(&mut store, predicate, &mut substitution)?;
    assert!(assumptions.prove(&store, &mut context, &goal)?.is_none());
    assert!(context.head(&store, unknown)?.is_none());
    Ok(())
}
