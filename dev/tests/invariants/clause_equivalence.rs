//! Duplicate clause keys preserve captures, correlations and nested binder ownership.
use crate::types::{
    GenericArgs, ItemId, Kind, Primitive, RegionId, Store, TypeId,
    arguments::Argument,
    parameters::{
        clauses::{Clause, Quantified},
        predicates::{Bound, Predicate},
    },
    projection, traits,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn appended_binders_keep_the_principals_slots_fixed() -> TestResult {
    let mut store = Store::default();
    let mut make = |count, slots: [u16; 2]| -> TestResult<_> {
        let regions = slots
            .map(|slot| RegionId::bound(0, slot))
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        let Clause::ForAll(id) = tuple(&mut store, count, &regions)? else {
            return Err("missing quantified test input".into());
        };
        Ok(id)
    };
    let left = make(3, [0, 2])?;
    let equivalent = make(5, [0, 4])?;
    let different = make(5, [1, 4])?;
    assert!(store.clause_comparison(left)?.0 == store.clause_comparison(different)?.0);
    let key = store.clause_comparison_preserving(left, 2)?.0;
    assert!(key == store.clause_comparison_preserving(equivalent, 2)?.0);
    assert!(key != store.clause_comparison_preserving(different, 2)?.0);
    assert_eq!(store.clause_comparison_preserving(left, 2)?.1, 1);
    assert!(store.clause_comparison_preserving(left, 4).is_err());
    assert_eq!(
        store
            .quantified(left)
            .ok_or("missing original binder")?
            .bound_regions,
        3
    );
    Ok(())
}

fn reference(store: &mut Store, region: RegionId) -> Result<TypeId, &'static str> {
    store.intern(Kind::Reference {
        mutable: false,
        region,
        referent: TypeId::primitive(Primitive::U8),
    })
}
fn clause(store: &mut Store, count: u32, predicate: Predicate) -> TestResult<Clause> {
    Ok(Clause::ForAll(store.intern_quantified(Quantified {
        bound_regions: count,
        predicate,
    })?))
}
fn tuple(store: &mut Store, count: u32, regions: &[RegionId]) -> TestResult<Clause> {
    let fields = regions
        .iter()
        .map(|&region| reference(store, region))
        .collect::<Result<Vec<_>, _>>()?;
    let fields = store.intern_list(&fields)?;
    let ty = store.intern(Kind::Tuple(fields))?;
    clause(
        store,
        count,
        Predicate {
            ty,
            bound: Bound::Copy,
        },
    )
}

#[test]
fn clause_comparison_preserves_original_slots_and_shares_only_derived_keys() -> TestResult {
    let mut store = Store::default();
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    let captured = RegionId::bound(1, 0)?;
    let left = tuple(&mut store, 2, &[a, b, captured, RegionId(70)])?;
    let right = tuple(
        &mut store,
        5,
        &[
            RegionId::bound(0, 4)?,
            RegionId::bound(0, 2)?,
            captured,
            RegionId(70),
        ],
    )?;
    assert_ne!(left, right);
    assert!(store.same_clause(left, right)?.0);
    let mut keys = crate::support::hash::HashSet::default();
    for clause in [left, right] {
        let Clause::ForAll(id) = clause else {
            return Err("missing quantified comparison input".into());
        };
        keys.insert(store.clause_comparison(id)?.0);
    }
    assert_eq!(keys.len(), 1);
    let warm_types = store.len();
    for _ in 0..100 {
        assert_eq!(store.same_clause(left, right)?, (true, 2));
    }
    assert_eq!(store.len(), warm_types);
    let Clause::ForAll(id) = right else {
        return Err("missing original clause".into());
    };
    assert_eq!(
        store
            .quantified(id)
            .ok_or("missing original binder")?
            .bound_regions,
        5
    );
    for regions in [
        [a, a, captured, RegionId(70)],
        [a, b, RegionId::bound(1, 1)?, RegionId(70)],
        [a, b, captured, RegionId(71)],
    ] {
        let other = tuple(&mut store, 2, &regions)?;
        assert!(!store.same_clause(left, other)?.0);
    }
    Ok(())
}

fn callable(store: &mut Store, slot: u16, nested_subject: bool) -> TestResult<Clause> {
    let outer = reference(store, RegionId::bound(u16::from(!nested_subject), slot)?)?;
    let inner = reference(store, RegionId::bound(0, 0)?)?;
    let capture = reference(store, RegionId::bound(2, 0)?)?;
    let parameters = store.intern_list(&[outer, inner, capture])?;
    let ty = store.intern(Kind::Function {
        parameters,
        result: inner,
        bound_regions: 1,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    })?;
    clause(
        store,
        u32::from(slot) + 1,
        Predicate {
            ty,
            bound: Bound::Copy,
        },
    )
}

#[test]
fn clause_comparison_keeps_nested_locals_and_outer_captures_rigid() -> TestResult {
    let mut store = Store::default();
    let left = callable(&mut store, 0, false)?;
    let reordered = callable(&mut store, 4, false)?;
    let nested = callable(&mut store, 0, true)?;
    assert!(store.same_clause(left, reordered)?.0);
    assert!(!store.same_clause(left, nested)?.0);
    Ok(())
}

fn equality(store: &mut Store, slots: [u16; 3]) -> TestResult<Clause> {
    store.declare_projection(
        ItemId(2),
        projection::Schema {
            trait_item: ItemId(1),
            sized: true,
            copies: crate::types::copy::Properties::default(),
        },
    )?;
    let arguments = store.intern_arguments(&[
        Argument::Lifetime(RegionId::bound(0, slots[0])?),
        Argument::Lifetime(RegionId::bound(0, slots[1])?),
    ])?;
    let trait_application = store.intern_trait_application(traits::Application {
        item: ItemId(1),
        arguments,
    })?;
    let ty = store.intern(Kind::Projection(projection::Application {
        item: ItemId(2),
        subject: TypeId::primitive(Primitive::U8),
        trait_application,
        arguments: GenericArgs::EMPTY,
    }))?;
    let output = reference(store, RegionId::bound(0, slots[2])?)?;
    clause(
        store,
        5,
        Predicate {
            ty,
            bound: Bound::ProjectionEqual(output),
        },
    )
}

#[test]
fn clause_comparison_includes_the_equality_result_correlation() -> TestResult {
    let mut store = Store::default();
    let left = equality(&mut store, [0, 1, 0])?;
    let same = equality(&mut store, [4, 2, 4])?;
    let changed = equality(&mut store, [4, 2, 2])?;
    assert!(store.same_clause(left, same)?.0);
    assert!(!store.same_clause(left, changed)?.0);
    let Clause::ForAll(id) = left else {
        return Err("missing quantified equality".into());
    };
    let ordinary = Clause::Predicate(
        store
            .quantified(id)
            .ok_or("missing equality body")?
            .predicate,
    );
    assert!(!store.same_clause(left, ordinary)?.0);
    Ok(())
}
