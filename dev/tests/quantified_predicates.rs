//! Whole-predicate binders preserve correlations without granting solver evidence.
use rustgm::types::{
    GenericArgs, ItemId, Kind, Primitive, RegionId, Store, TypeId,
    arguments::{Argument, ConstKind, Constant},
    parameters::{
        Parameter, ParameterKind,
        clauses::{Clause, Quantified, QuantifiedId},
        predicates::{Bound, Predicate},
    },
    projection::{self, Schema},
    substitute, traits,
};

const BYTE: TypeId = TypeId::primitive(Primitive::U8);
const SIZE: TypeId = TypeId::primitive(Primitive::Usize);

fn reference(store: &mut Store, region: RegionId, ty: TypeId) -> Result<TypeId, &'static str> {
    store.intern(Kind::Reference {
        mutable: false,
        region,
        referent: ty,
    })
}
fn function(store: &mut Store, inputs: &[TypeId], result: TypeId) -> Result<TypeId, &'static str> {
    let parameters = store.intern_list(inputs)?;
    store.intern(Kind::Function {
        parameters,
        result,
        bound_regions: 1,
        unsafe_: false,
        convention: rustgm::types::callable::Convention::RUST,
    })
}
fn trait_bound(store: &mut Store, arguments: &[Argument]) -> Result<Bound, &'static str> {
    let arguments = store.intern_arguments(arguments)?;
    Ok(Bound::Trait(store.intern_trait_application(
        traits::Application {
            item: ItemId(7),
            arguments,
        },
    )?))
}
fn quantified(store: &mut Store, count: u32, predicate: Predicate) -> Result<Clause, &'static str> {
    Ok(Clause::ForAll(store.intern_quantified(Quantified {
        bound_regions: count,
        predicate,
    })?))
}
fn body(store: &Store, clause: Clause) -> Result<Quantified, &'static str> {
    let Clause::ForAll(id) = clause else {
        return Err("lost quantified clause");
    };
    store.quantified(id).ok_or("missing quantified body")
}

#[test]
fn canonical_binders_validate_all_local_uses_without_claiming_a_proof() -> Result<(), &'static str>
{
    let mut store = Store::default();
    let local = reference(&mut store, RegionId::bound(0, 1)?, BYTE)?;
    let predicate = Predicate {
        ty: local,
        bound: Bound::Sized,
    };
    assert!(quantified(&mut store, 1, predicate).is_err());
    let clause = quantified(&mut store, 2, predicate)?;
    assert_eq!(quantified(&mut store, 2, predicate)?, clause);
    assert_ne!(quantified(&mut store, 3, predicate)?, clause);
    assert!(quantified(&mut store, 65_537, predicate).is_err());
    assert!(store.quantified(QuantifiedId(u32::MAX)).is_none());
    let captured = reference(&mut store, RegionId::bound(1, 9)?, BYTE)?;
    let captured = Predicate {
        ty: captured,
        bound: Bound::Sized,
    };
    let captured = quantified(&mut store, 0, captured)?;
    assert_eq!(body(&store, captured)?.bound_regions, 0);
    let max = reference(&mut store, RegionId::bound(0, u16::MAX)?, BYTE)?;
    quantified(
        &mut store,
        65_536,
        Predicate {
            ty: max,
            bound: Bound::Sized,
        },
    )?;
    for predicate in [
        Predicate {
            ty: TypeId(u32::MAX),
            bound: Bound::Sized,
        },
        Predicate {
            ty: BYTE,
            bound: Bound::Trait(traits::ApplicationId(u32::MAX)),
        },
        Predicate {
            ty: BYTE,
            bound: Bound::ProjectionEqual(TypeId(u32::MAX)),
        },
        Predicate {
            ty: BYTE,
            bound: Bound::Outlives(RegionId::bound(0, 2)?),
        },
    ] {
        assert!(quantified(&mut store, 2, predicate).is_err());
    }
    let nested_local = reference(&mut store, RegionId::bound(0, 0)?, BYTE)?;
    let nested_capture = reference(&mut store, RegionId::bound(1, 1)?, BYTE)?;
    let signature = function(&mut store, &[nested_local], nested_capture)?;
    let bound = trait_bound(&mut store, &[Argument::Type(signature)])?;
    assert!(quantified(&mut store, 1, Predicate { ty: BYTE, bound }).is_err());
    quantified(&mut store, 2, Predicate { ty: BYTE, bound })?;
    Ok(())
}

#[test]
fn joint_substitution_preserves_quantified_subject_and_complete_mixed_arguments()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let owner = ItemId(11);
    let parameter = store.intern_parameter(Parameter {
        owner,
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let ty = store.intern(Kind::Parameter(parameter))?;
    let parameter = store.intern_parameter(Parameter {
        owner,
        index: 1,
        kind: ParameterKind::Const { ty: SIZE },
    })?;
    let length = store.intern_const(ConstKind::Parameter(parameter))?;
    let array = store.intern(Kind::ArrayTerm {
        element: ty,
        length,
    })?;
    let subject = reference(&mut store, RegionId::bound(0, 1)?, array)?;
    let local = reference(&mut store, RegionId::bound(0, 0)?, ty)?;
    let inner_capture = reference(&mut store, RegionId::bound(1, 1)?, ty)?;
    let outer_capture = reference(&mut store, RegionId::bound(2, 0)?, ty)?;
    let nested = function(&mut store, &[local, inner_capture], outer_capture)?;
    let bound = trait_bound(
        &mut store,
        &[
            Argument::Lifetime(RegionId::bound(0, 0)?),
            Argument::Lifetime(RegionId::bound(1, 0)?),
            Argument::Type(nested),
            Argument::Const(length),
        ],
    )?;
    let clause = quantified(&mut store, 2, Predicate { ty: subject, bound })?;
    let replacement = reference(&mut store, RegionId::bound(0, 3)?, BYTE)?;
    let length = store.intern_constant(Constant { ty: SIZE, bits: 4 })?;
    let arguments =
        store.intern_arguments(&[Argument::Type(replacement), Argument::Const(length)])?;
    let mut fold =
        substitute::Bound::joint_captured(&store, owner, arguments, vec![RegionId::bound(0, 5)?])?;
    let actual = fold.clause(&mut store, clause)?;
    let shifted = reference(&mut store, RegionId::bound(1, 3)?, BYTE)?;
    let array = store.intern(Kind::Array {
        element: shifted,
        length: 4,
    })?;
    let subject = reference(&mut store, RegionId::bound(0, 1)?, array)?;
    let shifted = reference(&mut store, RegionId::bound(2, 3)?, BYTE)?;
    let local = reference(&mut store, RegionId::bound(0, 0)?, shifted)?;
    let inner_capture = reference(&mut store, RegionId::bound(1, 1)?, shifted)?;
    let outer_capture = reference(&mut store, RegionId::bound(2, 5)?, shifted)?;
    let nested = function(&mut store, &[local, inner_capture], outer_capture)?;
    let bound = trait_bound(
        &mut store,
        &[
            Argument::Lifetime(RegionId::bound(0, 0)?),
            Argument::Lifetime(RegionId::bound(1, 5)?),
            Argument::Type(nested),
            Argument::Const(length),
        ],
    )?;
    assert_eq!(
        body(&store, actual)?,
        Quantified {
            bound_regions: 2,
            predicate: Predicate { ty: subject, bound },
        }
    );
    let steps = fold.steps;
    assert_eq!(fold.clause(&mut store, clause)?, actual);
    assert_eq!(fold.steps, steps);
    Ok(())
}

#[test]
fn ordinary_and_quantified_application_caches_never_alias_in_either_order()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let subject = reference(&mut store, RegionId::bound(0, 0)?, BYTE)?;
    let bound = trait_bound(
        &mut store,
        &[
            Argument::Lifetime(RegionId::bound(0, 0)?),
            Argument::Lifetime(RegionId::bound(1, 0)?),
        ],
    )?;
    let predicate = Predicate { ty: subject, bound };
    let universal = quantified(&mut store, 1, predicate)?;
    let ordinary = Clause::Predicate(predicate);
    let outer_subject = reference(&mut store, RegionId::bound(0, 5)?, BYTE)?;
    let outer_bound = trait_bound(
        &mut store,
        &[
            Argument::Lifetime(RegionId::bound(0, 5)?),
            Argument::Lifetime(RegionId::bound(0, 0)?),
        ],
    )?;
    let expected_ordinary = Clause::Predicate(Predicate {
        ty: outer_subject,
        bound: outer_bound,
    });
    let bound = trait_bound(
        &mut store,
        &[
            Argument::Lifetime(RegionId::bound(0, 0)?),
            Argument::Lifetime(RegionId::bound(1, 5)?),
        ],
    )?;
    let expected_universal = quantified(&mut store, 1, Predicate { ty: subject, bound })?;
    for order in [[ordinary, universal], [universal, ordinary]] {
        let mut fold = substitute::Bound::new(vec![RegionId::bound(0, 5)?]);
        for clause in order.into_iter().cycle().take(8) {
            assert_eq!(
                fold.clause(&mut store, clause)?,
                if clause == ordinary {
                    expected_ordinary
                } else {
                    expected_universal
                }
            );
        }
    }
    Ok(())
}

#[test]
fn quantified_equality_maps_projection_inputs_and_result_without_capture()
-> Result<(), &'static str> {
    let mut store = Store::default();
    store.declare_projection(
        ItemId(8),
        Schema {
            trait_item: ItemId(7),
            sized: true,
            copies: rustgm::types::copy::Properties::default(),
        },
    )?;
    let subject = reference(&mut store, RegionId::bound(0, 0)?, BYTE)?;
    let Bound::Trait(trait_application) =
        trait_bound(&mut store, &[Argument::Lifetime(RegionId::bound(1, 0)?)])?
    else {
        return Err("missing trait");
    };
    let projected = store.intern(Kind::Projection(projection::Application {
        item: ItemId(8),
        subject,
        trait_application,
        arguments: GenericArgs::EMPTY,
    }))?;
    let result = reference(&mut store, RegionId::bound(1, 0)?, subject)?;
    let clause = quantified(
        &mut store,
        1,
        Predicate {
            ty: projected,
            bound: Bound::ProjectionEqual(result),
        },
    )?;
    let mut fold = substitute::Bound::new(vec![RegionId(19)]);
    let actual = fold.clause(&mut store, clause)?;
    let Bound::Trait(trait_application) =
        trait_bound(&mut store, &[Argument::Lifetime(RegionId(19))])?
    else {
        return Err("missing trait");
    };
    let projected = store.intern(Kind::Projection(projection::Application {
        item: ItemId(8),
        subject,
        trait_application,
        arguments: GenericArgs::EMPTY,
    }))?;
    let result = reference(&mut store, RegionId(19), subject)?;
    assert_eq!(
        body(&store, actual)?.predicate,
        Predicate {
            ty: projected,
            bound: Bound::ProjectionEqual(result),
        }
    );
    for (written, expected) in [
        (RegionId::bound(0, 0)?, RegionId::bound(0, 0)?),
        (RegionId::bound(1, 0)?, RegionId(19)),
    ] {
        let clause = quantified(
            &mut store,
            1,
            Predicate {
                ty: subject,
                bound: Bound::Outlives(written),
            },
        )?;
        let actual = fold.clause(&mut store, clause)?;
        assert_eq!(
            body(&store, actual)?.predicate,
            Predicate {
                ty: subject,
                bound: Bound::Outlives(expected),
            }
        );
    }
    Ok(())
}

#[test]
fn empty_quantifier_retains_capture_depth_and_missing_arguments_are_errors()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let subject = reference(&mut store, RegionId::bound(1, 0)?, BYTE)?;
    let clause = quantified(
        &mut store,
        0,
        Predicate {
            ty: subject,
            bound: Bound::Copy,
        },
    )?;
    assert!(
        substitute::Bound::new(vec![])
            .clause(&mut store, clause)
            .is_err()
    );
    let mut fold = substitute::Bound::new(vec![RegionId::bound(0, 4)?]);
    let actual = fold.clause(&mut store, clause)?;
    let subject = reference(&mut store, RegionId::bound(1, 4)?, BYTE)?;
    assert_eq!(
        body(&store, actual)?,
        Quantified {
            bound_regions: 0,
            predicate: Predicate {
                ty: subject,
                bound: Bound::Copy
            },
        }
    );
    assert!(
        fold.clause(&mut store, Clause::ForAll(QuantifiedId(u32::MAX)))
            .is_err()
    );
    Ok(())
}

#[test]
fn repeated_wide_quantified_applications_reuse_completed_substitution_work()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let bound = trait_bound(
        &mut store,
        &vec![Argument::Lifetime(RegionId::bound(1, 0)?); 2048],
    )?;
    let clause = quantified(&mut store, 1, Predicate { ty: BYTE, bound })?;
    let mut fold = substitute::Bound::new(vec![RegionId::STATIC]);
    let actual = fold.clause(&mut store, clause)?;
    let steps = fold.steps;
    assert!(steps < 2100, "{steps}");
    for _ in 0..1024 {
        assert_eq!(fold.clause(&mut store, clause)?, actual);
    }
    assert_eq!(fold.steps, steps);
    let mut exhausted = substitute::Bound::new(vec![RegionId::STATIC]);
    exhausted.steps = 1_000_000;
    assert!(exhausted.clause(&mut store, clause).is_err());
    Ok(())
}
