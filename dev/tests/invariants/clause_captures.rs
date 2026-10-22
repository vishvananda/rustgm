//! The same canonical graph has different captures inside and outside a quantified binder.
use super::Uses;
use crate::support::hash::HashSet;
use crate::types::{
    ItemId, Kind, RegionId, Store, TypeId,
    arguments::Argument,
    parameters::{
        clauses::{Clause, Quantified},
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

#[test]
fn application_memoization_keeps_ordinary_and_quantified_depths_separate()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let arguments = store.intern_arguments(&[
        Argument::Lifetime(RegionId::bound(0, 1)?),
        Argument::Lifetime(RegionId::bound(1, 7)?),
    ])?;
    let application = store.intern_trait_application(Application {
        item: ItemId(1),
        arguments,
    })?;
    let predicate = Predicate {
        ty: TypeId::UNIT,
        bound: Bound::Trait(application),
    };
    let quantified = quantify(&mut store, 2, predicate)?;
    let ordinary = Clause::Predicate(predicate);
    for (first, second, expected) in [(ordinary, quantified, 1), (quantified, ordinary, 7)] {
        let mut uses = Uses::default();
        uses.clause(&store, first)?;
        assert_eq!(uses.indices, HashSet::from_iter([expected]));
        uses.clause(&store, second)?;
        assert_eq!(uses.indices, HashSet::from_iter([1, 7]));
        let steps = uses.steps;
        uses.clause(&store, first)?;
        uses.clause(&store, second)?;
        assert_eq!(uses.steps, steps);
    }
    Ok(())
}

#[test]
fn whole_clause_occurrences_keep_callable_locals_and_enclosing_captures_distinct()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let local = reference(&mut store, 0, 0)?;
    let quantified_local = reference(&mut store, 1, 1)?;
    let captured = reference(&mut store, 2, 5)?;
    let parameters = store.intern_list(&[local, quantified_local, captured])?;
    let function = store.intern(Kind::Function {
        parameters,
        result: local,
        bound_regions: 1,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    })?;
    let equality = reference(&mut store, 1, 7)?;
    let clause = quantify(
        &mut store,
        2,
        Predicate {
            ty: function,
            bound: Bound::ProjectionEqual(equality),
        },
    )?;
    let mut uses = Uses::default();
    uses.clause(&store, clause)?;
    assert_eq!(uses.indices, HashSet::from_iter([5, 7]));
    let outlives = quantify(
        &mut store,
        2,
        Predicate {
            ty: function,
            bound: Bound::Outlives(RegionId::bound(1, 3)?),
        },
    )?;
    uses.clause(&store, outlives)?;
    assert_eq!(uses.indices, HashSet::from_iter([3, 5, 7]));
    assert!(clause.ordinary().is_err());
    Ok(())
}
