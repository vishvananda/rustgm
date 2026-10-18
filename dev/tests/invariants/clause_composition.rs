//! Structural composition preserves complete formulas, without proving their predicates.
use crate::types::{
    ItemId, Kind, Primitive, RegionId, Store, TypeId,
    arguments::{Argument, Constant},
    parameters::{
        clauses::{Clause, Quantified, Quantifier},
        predicates::{Bound, Predicate},
    },
    traits::Application,
};

fn reference(store: &mut Store, depth: u16, slot: u16) -> Result<TypeId, &'static str> {
    store.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(depth, slot)?,
        referent: TypeId::UNIT,
    })
}
fn quantify(store: &mut Store, count: u32, predicate: Predicate) -> Result<Clause, &'static str> {
    Ok(Clause::ForAll(store.intern_quantified(Quantified {
        bound_regions: count,
        predicate,
    })?))
}
fn body(store: &Store, clause: Clause) -> Result<Quantified, &'static str> {
    let Clause::ForAll(id) = clause else {
        return Err("missing composed quantifier");
    };
    store.quantified(id).ok_or("invalid composed identity")
}
fn callable(store: &mut Store, regions: &[(u16, u16)]) -> Result<TypeId, &'static str> {
    let mut parameters = Vec::new();
    for &(depth, slot) in regions {
        parameters.push(reference(store, depth, slot)?);
    }
    let parameters = store.intern_list(&parameters)?;
    store.intern(Kind::Function {
        parameters,
        result: TypeId::UNIT,
        bound_regions: 1,
        unsafe_: false,
    })
}
fn predicate(store: &mut Store, ty: TypeId, args: &[Argument]) -> Result<Predicate, &'static str> {
    let arguments = store.intern_arguments(args)?;
    let id = store.intern_trait_application(Application {
        item: ItemId(7),
        arguments,
    })?;
    Ok(Predicate {
        ty,
        bound: Bound::Trait(id),
    })
}

#[test]
fn composition_preserves_subject_arguments_constants_and_nested_callable_locals()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let subject = reference(&mut store, 1, 1)?;
    let function = callable(&mut store, &[(0, 0), (1, 0), (2, 1), (3, 3)])?;
    let constant = store.intern_constant(Constant {
        ty: TypeId::primitive(Primitive::Usize),
        bits: 7,
    })?;
    let value = predicate(
        &mut store,
        subject,
        &[
            Argument::Lifetime(RegionId::bound(0, 0)?),
            Argument::Lifetime(RegionId::bound(1, 0)?),
            Argument::Lifetime(RegionId::bound(2, 3)?),
            Argument::Type(function),
            Argument::Const(constant),
        ],
    )?;
    let input = quantify(&mut store, 1, value)?;
    let mut binder = Quantifier::new(2)?;
    let output = binder.clause(&mut store, input)?;
    let subject = reference(&mut store, 0, 1)?;
    let function = callable(&mut store, &[(0, 0), (1, 2), (1, 1), (2, 3)])?;
    let expected = predicate(
        &mut store,
        subject,
        &[
            Argument::Lifetime(RegionId::bound(0, 2)?),
            Argument::Lifetime(RegionId::bound(0, 0)?),
            Argument::Lifetime(RegionId::bound(1, 3)?),
            Argument::Type(function),
            Argument::Const(constant),
        ],
    )?;
    assert_eq!(
        body(&store, output)?,
        Quantified {
            bound_regions: 3,
            predicate: expected
        }
    );
    assert_eq!(binder.clause(&mut store, input)?, output);
    Ok(())
}

#[test]
fn empty_binders_and_equality_results_keep_their_capture_depth() -> Result<(), &'static str> {
    let mut store = Store::default();
    let subject = reference(&mut store, 1, 0)?;
    let result = reference(&mut store, 2, 1)?;
    let inner = quantify(
        &mut store,
        0,
        Predicate {
            ty: subject,
            bound: Bound::ProjectionEqual(result),
        },
    )?;
    let output = Quantifier::new(1)?.clause(&mut store, inner)?;
    let subject = reference(&mut store, 0, 0)?;
    let result = reference(&mut store, 1, 1)?;
    assert_eq!(
        body(&store, output)?,
        Quantified {
            bound_regions: 1,
            predicate: Predicate {
                ty: subject,
                bound: Bound::ProjectionEqual(result)
            }
        }
    );
    let captured = reference(&mut store, 2, 0)?;
    let inner = quantify(
        &mut store,
        0,
        Predicate {
            ty: captured,
            bound: Bound::Sized,
        },
    )?;
    let composed = Quantifier::new(0)?.clause(&mut store, inner)?;
    let captured = reference(&mut store, 1, 0)?;
    let ordinary = Quantifier::new(0)?.clause(
        &mut store,
        Clause::Predicate(Predicate {
            ty: captured,
            bound: Bound::Sized,
        }),
    )?;
    assert_eq!(composed, ordinary);
    Ok(())
}

#[test]
fn invalid_parent_slots_cannot_become_valid_inner_slots() -> Result<(), &'static str> {
    let mut store = Store::default();
    for (outer, inner_count, slot) in [(0, 1, 0), (1, 2, 1), (2, 1, 2)] {
        let subject = reference(&mut store, 1, slot)?;
        let input = quantify(
            &mut store,
            inner_count,
            Predicate {
                ty: subject,
                bound: Bound::Sized,
            },
        )?;
        assert_eq!(
            Quantifier::new(outer)?.clause(&mut store, input),
            Err("composed clause references an unbound outer lifetime")
        );
        let input = quantify(
            &mut store,
            inner_count,
            Predicate {
                ty: TypeId::UNIT,
                bound: Bound::Outlives(RegionId::bound(1, slot)?),
            },
        )?;
        assert!(Quantifier::new(outer)?.clause(&mut store, input).is_err());
    }
    // Parent validation includes arguments and equality results, even when
    // the ordinary subject has no lifetime occurrences of its own.
    let invalid = reference(&mut store, 1, 1)?;
    let argument = predicate(&mut store, TypeId::UNIT, &[Argument::Type(invalid)])?;
    for value in [
        argument,
        Predicate {
            ty: TypeId::UNIT,
            bound: Bound::ProjectionEqual(invalid),
        },
    ] {
        let input = quantify(&mut store, 1, value)?;
        assert_eq!(
            Quantifier::new(1)?.clause(&mut store, input),
            Err("composed clause references an unbound outer lifetime")
        );
    }
    let empty = quantify(
        &mut store,
        0,
        Predicate {
            ty: TypeId::UNIT,
            bound: Bound::Sized,
        },
    )?;
    assert!(Quantifier::new(65_537).is_err());
    let mut largest = Quantifier::new(65_536)?;
    let maximal = largest.clause(&mut store, empty)?;
    assert_eq!(body(&store, maximal)?.bound_regions, 65_536);
    let one = quantify(
        &mut store,
        1,
        Predicate {
            ty: TypeId::UNIT,
            bound: Bound::Sized,
        },
    )?;
    assert!(largest.clause(&mut store, one).is_err());
    Ok(())
}

#[test]
fn distinct_composed_clauses_share_validation_and_opening_of_growing_tails()
-> Result<(), &'static str> {
    let mut previous = 0;
    for count in [64, 128, 256, 512] {
        let mut store = Store::default();
        let mut tail = TypeId::UNIT;
        let mut inputs = Vec::new();
        for _ in 0..count {
            tail = store.intern(Kind::Reference {
                mutable: false,
                region: RegionId::bound(1, 0)?,
                referent: tail,
            })?;
            inputs.push(quantify(
                &mut store,
                1,
                Predicate {
                    ty: tail,
                    bound: Bound::Outlives(RegionId::bound(0, 0)?),
                },
            )?);
        }
        let initial = store.len();
        let before = store
            .arguments
            .as_ref()
            .and_then(|s| s.clauses.as_ref())
            .ok_or("missing clause storage")?
            .slots
            .steps;
        let mut binder = Quantifier::new(1)?;
        for input in inputs {
            binder.clause(&mut store, input)?;
        }
        let validation = store
            .arguments
            .as_ref()
            .and_then(|s| s.clauses.as_ref())
            .ok_or("missing clause storage")?
            .slots
            .steps
            - before;
        println!(
            "clause-composition count={count} steps={} validation={validation} types={}",
            binder.steps,
            store.len() - initial
        );
        assert!(binder.steps + validation <= 20 * count + 32);
        assert!(binder.steps <= 12 * count + 16);
        assert!(store.len() - initial <= count);
        assert!(previous == 0 || binder.steps <= 2 * previous + 16);
        previous = binder.steps;
    }
    Ok(())
}
