use super::*;
use crate::types::{Kind, RegionId, TypeId, parameters::predicates::Bound};

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
