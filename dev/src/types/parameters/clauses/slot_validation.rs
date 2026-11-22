use super::*;
use crate::types::{Kind, RegionId, TypeId, parameters::predicates::Bound};

#[test]
fn definition_satisfaction_distinguishes_unused_nonempty_and_empty_quantifiers()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let predicate = Predicate {
        ty: TypeId::UNIT,
        bound: Bound::Copy,
    };
    assert!(Clause::Predicate(predicate).requires_definition_proof(&store)?);
    for count in [0, 1, 3] {
        let id = store.intern_quantified(Quantified {
            bound_regions: count,
            predicate,
        })?;
        assert_eq!(
            Clause::ForAll(id).requires_definition_proof(&store)?,
            count == 0
        );
    }
    let ty = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(1, 0)?,
        referent: TypeId::UNIT,
    })?;
    let id = store.intern_quantified(Quantified {
        bound_regions: 0,
        predicate: Predicate { ty, ..predicate },
    })?;
    assert!(!Clause::ForAll(id).requires_definition_proof(&store)?);
    Ok(())
}

#[test]
fn closed_binder_flags_reach_subjects_trait_arguments_and_equality_results()
-> Result<(), &'static str> {
    use crate::types::{ItemId, arguments::Argument, traits::Application};
    let mut store = Store::default();
    let parameters = store.intern_list(&[])?;
    for count in [0, 1] {
        let function = store.intern(Kind::Function {
            parameters,
            result: TypeId::UNIT,
            bound_regions: count,
            unsafe_: false,
            convention: crate::types::callable::Convention::RUST,
        })?;
        assert_eq!(store.escaping_binders(function), Some(0));
        let arguments = store.intern_arguments(&[Argument::Type(function)])?;
        let application = store.intern_trait_application(Application {
            item: ItemId(99),
            arguments,
        })?;
        for predicate in [
            Predicate {
                ty: function,
                bound: Bound::Copy,
            },
            Predicate {
                ty: TypeId::UNIT,
                bound: Bound::Trait(application),
            },
            Predicate {
                ty: TypeId::UNIT,
                bound: Bound::ProjectionEqual(function),
            },
        ] {
            assert_eq!(predicate.requires_definition_proof(&store)?, count == 0);
        }
    }
    Ok(())
}

#[test]
fn distinct_clauses_share_slot_summaries_instead_of_rescanning_growing_tails()
-> Result<(), &'static str> {
    let mut previous = 0;
    for count in [64, 128, 256, 512] {
        let mut store = Store::default();
        let mut ty = TypeId::UNIT;
        for _ in 0..count {
            ty = store.intern(Kind::Reference {
                mutable: false,
                region: RegionId::bound(0, 0)?,
                referent: ty,
            })?;
            let value = Quantified {
                bound_regions: 1,
                predicate: Predicate {
                    ty,
                    bound: Bound::Sized,
                },
            };
            let id = store.intern_quantified(value)?;
            assert_eq!(store.intern_quantified(value)?, id);
            assert!(
                store
                    .intern_quantified(Quantified {
                        bound_regions: 0,
                        ..value
                    })
                    .is_err()
            );
        }
        let storage = store
            .arguments
            .as_ref()
            .and_then(|s| s.clauses.as_ref())
            .ok_or("missing clauses")?;
        let steps = storage.slots.steps;
        assert!(steps <= count * 4, "{count}: {steps}");
        assert!(
            previous == 0 || steps <= previous * 2 + 4,
            "{count}: {previous} -> {steps}"
        );
        previous = steps;
    }
    Ok(())
}

#[test]
fn slot_summary_checks_both_sides_and_does_not_depend_on_previous_binder_counts()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let ty = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(0, 0)?,
        referent: TypeId::UNIT,
    })?;
    let high = Predicate {
        ty,
        bound: Bound::Outlives(RegionId::bound(0, 7)?),
    };
    store.intern_quantified(Quantified {
        bound_regions: 8,
        predicate: high,
    })?;
    assert!(
        store
            .intern_quantified(Quantified {
                bound_regions: 7,
                predicate: high
            })
            .is_err()
    );
    store.intern_quantified(Quantified {
        bound_regions: 1,
        predicate: Predicate {
            ty,
            bound: Bound::Sized,
        },
    })?;
    Ok(())
}

#[test]
fn specializing_quantified_clauses_preserves_slots_without_rescanning() -> Result<(), &'static str>
{
    use crate::types::{
        ItemId,
        arguments::Argument,
        parameters::{Parameter, ParameterKind},
        substitute::Bound as Substitute,
    };

    let mut store = Store::default();
    let parameter = store.intern_parameter(Parameter {
        owner: ItemId(3),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let referent = store.intern(Kind::Parameter(parameter))?;
    let local = RegionId::bound(0, 0)?;
    let ty = store.intern(Kind::Reference {
        mutable: false,
        region: local,
        referent,
    })?;
    let source = store.intern_quantified(Quantified {
        bound_regions: 1,
        predicate: Predicate {
            ty,
            bound: Bound::Sized,
        },
    })?;
    for slot in 0..128 {
        let replacement = store.intern(Kind::Reference {
            mutable: false,
            region: RegionId::bound(0, slot)?,
            referent: TypeId::UNIT,
        })?;
        let arguments = [(parameter, Argument::Type(replacement))];
        for mut fold in [
            Substitute::parameters(&store, &arguments)?,
            Substitute::parameters_captured(&store, &arguments, Vec::new())?,
        ] {
            let Clause::ForAll(id) = fold.clause(&mut store, Clause::ForAll(source))? else {
                return Err("substitution lost the quantified binder");
            };
            let value = store.quantified(id).ok_or("missing specialized clause")?;
            assert_eq!(value.bound_regions, 1);
            let shifted = store.intern(Kind::Reference {
                mutable: false,
                region: RegionId::bound(1, slot)?,
                referent: TypeId::UNIT,
            })?;
            assert_eq!(
                store.kind(value.predicate.ty),
                Some(Kind::Reference {
                    mutable: false,
                    region: local,
                    referent: shifted,
                })
            );
        }
    }
    let storage = store
        .arguments
        .as_ref()
        .and_then(|s| s.clauses.as_ref())
        .ok_or("missing clauses")?;
    assert!(storage.slots.steps <= 8);
    Ok(())
}
