//! The assumption index must bound unrelated candidates without guessing unknown Self.
use super::{Assumptions, Bound, Context, ItemId, Predicate, Store};
use crate::types::{
    Kind, Primitive, TypeId,
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
            .prove(&mut store, &mut context, &goal)?
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
    assert!(
        assumptions
            .prove(&mut store, &mut context, &goal)?
            .is_none()
    );
    assert!(context.head(&store, unknown)?.is_none());
    Ok(())
}

fn projected_alternatives() -> Result<(Store, Predicate, Predicate), Box<dyn std::error::Error>> {
    use crate::types::{GenericArgs, copy::Properties, projection};
    let mut store = Store::default();
    let parameter = store.intern_parameter(Parameter {
        owner: ItemId(2),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let subject = store.intern(Kind::Parameter(parameter))?;
    let trait_application = store.intern_trait_application(Application {
        item: ItemId(3),
        arguments: GenericArgs::EMPTY,
    })?;
    store.declare_projection(
        ItemId(4),
        projection::Schema {
            trait_item: ItemId(3),
            sized: true,
            copies: Properties::default(),
        },
    )?;
    let projection = store.intern(Kind::Projection(projection::Application {
        item: ItemId(4),
        subject,
        trait_application,
        arguments: GenericArgs::EMPTY,
    }))?;
    let mut predicate = |ty| -> Result<Predicate, &'static str> {
        let arguments = store.intern_arguments(&[Argument::Type(ty)])?;
        Ok(Predicate {
            ty: subject,
            bound: Bound::Trait(store.intern_trait_application(Application {
                item: ItemId(1),
                arguments,
            })?),
        })
    };
    let opaque = predicate(projection)?;
    let exact = predicate(TypeId::primitive(Primitive::U8))?;
    Ok((store, opaque, exact))
}

#[test]
fn blocked_alternatives_do_not_hide_independent_evidence() -> Result<(), Box<dyn std::error::Error>>
{
    for reverse in [false, true] {
        let (mut store, opaque, exact) = projected_alternatives()?;
        let mut assumptions = Assumptions::default();
        for predicate in if reverse {
            [exact, opaque]
        } else {
            [opaque, exact]
        } {
            assumptions.insert(&store, predicate)?;
        }
        // Either rigid application has its own evidence; neither projection
        // equality nor a guessed normal form is needed to match that evidence.
        for predicate in [exact, opaque] {
            let mut context = Context::default();
            let evidence = assumptions
                .prove_canonical(&mut store, &mut context, predicate)?
                .ok_or("blocked alternative hid the caller's matching assumption")?;
            assert_eq!(evidence.assumption(), predicate);
            assert!(context.region_equalities.is_empty());
            assert!(context.steps < 200);
        }
    }
    Ok(())
}

#[test]
fn blocked_only_is_not_absence_or_permission() -> Result<(), Box<dyn std::error::Error>> {
    let (mut store, opaque, exact) = projected_alternatives()?;
    let mut assumptions = Assumptions::default();
    assumptions.insert(&store, opaque)?;
    let mut context = Context::default();
    assert!(matches!(
        assumptions.prove_canonical(&mut store, &mut context, exact),
        Err(crate::types::infer::Error::Normalization)
    ));
    assert!(context.region_equalities.is_empty());
    Ok(())
}

#[test]
fn unknown_argument_is_not_guessed_from_one_plain_alternative()
-> Result<(), Box<dyn std::error::Error>> {
    let (mut store, opaque, exact) = projected_alternatives()?;
    let mut assumptions = Assumptions::default();
    assumptions.insert(&store, opaque)?;
    assumptions.insert(&store, exact)?;
    let parameter = store.intern_parameter(Parameter {
        owner: ItemId(5),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let ty = store.intern(Kind::Parameter(parameter))?;
    let arguments = store.intern_arguments(&[Argument::Type(ty)])?;
    let predicate = Predicate {
        ty: exact.ty,
        bound: Bound::Trait(store.intern_trait_application(Application {
            item: ItemId(1),
            arguments,
        })?),
    };
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
    assert!(matches!(
        assumptions.select(&mut store, &mut context, &goal),
        Err(crate::types::infer::Error::Unresolved)
    ));
    assert!(context.head(&store, unknown)?.is_none());
    Ok(())
}

#[test]
fn only_selected_lifetime_equations_escape_the_probe() -> Result<(), Box<dyn std::error::Error>> {
    use crate::types::RegionId;
    let (mut store, opaque, exact) = projected_alternatives()?;
    let mut with_region = |predicate: Predicate, region| -> Result<Predicate, &'static str> {
        let Bound::Trait(id) = predicate.bound else {
            return Err("missing test trait application");
        };
        let application = store.trait_application(id).ok_or("missing test trait")?;
        let mut arguments = vec![Argument::Lifetime(region)];
        arguments.extend(
            store
                .arguments(application.arguments)
                .ok_or("missing arguments")?,
        );
        let arguments = store.intern_arguments(&arguments)?;
        Ok(Predicate {
            ty: predicate.ty,
            bound: Bound::Trait(store.intern_trait_application(Application {
                item: application.item,
                arguments,
            })?),
        })
    };
    let blocked = with_region(opaque, RegionId(1))?;
    let selected = with_region(exact, RegionId(2))?;
    let predicate = with_region(exact, RegionId(3))?;
    let mut assumptions = Assumptions::default();
    assumptions.insert(&store, blocked)?;
    assumptions.insert(&store, selected)?;
    let mut context = Context::default();
    let evidence = assumptions
        .prove_canonical(&mut store, &mut context, predicate)?
        .ok_or("missing selected lifetime evidence")?;
    assert_eq!(evidence.assumption(), selected);
    assert_eq!(context.region_equalities, [(RegionId(3), RegionId(2))]);
    assumptions.insert(&store, predicate)?;
    let mut context = Context::default();
    assert!(
        assumptions
            .prove_canonical(&mut store, &mut context, predicate)?
            .is_none()
    );
    assert!(context.region_equalities.is_empty());
    Ok(())
}
