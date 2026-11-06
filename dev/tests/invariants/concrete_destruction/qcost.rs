//! The same scoped matcher pays its cold storage and cannot import unpaid answers.
use super::*;
use crate::types::{
    RegionId,
    parameters::{
        Parameter, ParameterKind,
        clauses::{Clause, Quantified},
        environment::Abilities,
        predicates::{Bound, Predicate},
    },
};
fn subjects(
    types: &mut Store,
    a: RegionId,
    b: RegionId,
    ty: TypeId,
) -> Result<TypeId, &'static str> {
    let a = types.intern(Kind::Reference {
        mutable: false,
        region: a,
        referent: TypeId::UNIT,
    })?;
    let b = types.intern(Kind::Reference {
        mutable: false,
        region: b,
        referent: TypeId::UNIT,
    })?;
    let list = types.intern_list(&[a, b, ty])?;
    types.intern(Kind::Tuple(list))
}
fn fixture() -> Result<(Store, Arc<Environment>, TypeId, TypeId), &'static str> {
    let mut types = Store::default();
    let id = types.intern_parameter(Parameter {
        owner: ItemId(3),
        index: 0,
        kind: ParameterKind::Type { sized: false },
    })?;
    let ty = types.intern(Kind::Parameter(id))?;
    let region = RegionId::bound(0, 0)?;
    let pattern = subjects(&mut types, region, region, ty)?;
    let clause = Clause::ForAll(types.intern_quantified(Quantified {
        bound_regions: 1,
        predicate: Predicate {
            ty: pattern,
            bound: Bound::Copy,
        },
    })?);
    let environment = Arc::new(
        Environment::new(&types, &[(id, Abilities::default())])?.with_clauses(&types, &[clause])?,
    );
    let actual = subjects(&mut types, RegionId(7), RegionId(7), ty)?;
    let mismatch = subjects(&mut types, RegionId(7), RegionId(8), ty)?;
    Ok((types, environment, actual, mismatch))
}
#[test]
fn scoped_cold_warm_and_mismatch_queries_pay_the_same_cache_once() -> Result<(), &'static str> {
    let (types, environment, actual, mismatch) = fixture()?;
    let mut query = NeedsDrop::scoped_bounded([].into_iter(), environment, LIMIT)?;
    let before = query.retained_bytes();
    assert!(!query.demand_scoped_bounded(&types, &Definitions::default(), actual, LIMIT)?);
    assert!(query.demand_scoped_bounded(&types, &Definitions::default(), mismatch, LIMIT)?);
    let bytes = query.retained_bytes();
    let paid = query.scope.as_ref().ok_or("scope")?.builtin_bytes;
    assert!(paid > size_of::<crate::types::parameters::environment::BuiltinQuery>());
    assert!(bytes > before && bytes >= paid);
    let steps = query.steps;
    for _ in 0..32 {
        assert!(!query.demand_scoped_bounded(&types, &Definitions::default(), actual, LIMIT)?);
        assert!(query.demand_scoped_bounded(&types, &Definitions::default(), mismatch, LIMIT)?);
    }
    assert_eq!(query.steps, steps + 64);
    assert_eq!(query.retained_bytes(), bytes);
    assert_eq!(query.scope.as_ref().ok_or("scope")?.builtin_bytes, paid);
    Ok(())
}
#[test]
fn scoped_zero_partial_failure_and_unpaid_warm_cache_never_publish_answers()
-> Result<(), &'static str> {
    let (types, environment, actual, _) = fixture()?;
    for extra in [
        0,
        size_of::<crate::types::parameters::environment::BuiltinQuery>(),
    ] {
        let mut query = NeedsDrop::scoped_bounded([].into_iter(), Arc::clone(&environment), LIMIT)?;
        let limit = query.retained_bytes() + extra;
        let before = query.steps;
        assert_eq!(
            query.assumed_bounded(&types, actual, limit),
            Err("destruction query storage budget exceeded")
        );
        assert!(query.steps > before);
        let scope = query.scope.as_ref().ok_or("scope")?;
        assert_eq!(scope.builtins.is_none(), extra == 0);
        if scope.builtins.is_some() {
            let paid = scope.builtin_bytes;
            assert_eq!(
                query.assumed_bounded(&types, actual, LIMIT),
                Err("destruction query storage budget exceeded")
            );
            assert_eq!(query.scope.as_ref().ok_or("scope")?.builtin_bytes, paid);
        }
    }
    let mut unpaid = NeedsDrop::in_environment([], environment);
    assert!(!unpaid.demand(&types, &Definitions::default(), actual)?);
    assert!(unpaid.scope.as_ref().ok_or("scope")?.builtins.is_some());
    assert_eq!(
        unpaid.demand_scoped_bounded(&types, &Definitions::default(), actual, LIMIT),
        Err("bounded destruction cannot reuse an unpaid builtin query")
    );
    Ok(())
}
#[test]
fn scoped_foreign_or_later_unobserved_growth_cannot_reuse_paid_truth() -> Result<(), &'static str> {
    let (types, environment, actual, mismatch) = fixture()?;
    let mut paid = NeedsDrop::scoped_bounded([].into_iter(), environment, LIMIT)?;
    assert!(!paid.demand_scoped_bounded(&types, &Definitions::default(), actual, LIMIT)?);
    let foreign = Arc::new(Environment::new(&types, &[])?);
    let mut query = NeedsDrop::scoped_bounded([].into_iter(), foreign, LIMIT)?;
    {
        let source = paid.scope.as_mut().ok_or("scope")?;
        let target = query.scope.as_mut().ok_or("scope")?;
        target.builtins = source.builtins.take();
        target.builtin_bytes = source.builtin_bytes;
    }
    assert!(
        query
            .demand_scoped_bounded(&types, &Definitions::default(), actual, LIMIT)
            .is_err()
    );
    paid.scope.as_mut().ok_or("scope")?.builtins =
        query.scope.as_mut().ok_or("scope")?.builtins.take();
    assert!(paid.demand(&types, &Definitions::default(), mismatch)?);
    assert_eq!(
        paid.demand_scoped_bounded(&types, &Definitions::default(), actual, LIMIT),
        Err("bounded destruction cannot reuse an unpaid builtin query")
    );
    assert_eq!(size_of::<super::super::super::Scope>(), 64);
    Ok(())
}

#[test]
fn copy_seeds_are_scoped_prepaid_and_cannot_rewrite_completed_parents() -> Result<(), &'static str>
{
    let (mut types, environment, actual, mismatch) = fixture()?;
    let fields = types.intern_list(&[actual, TypeId::UNIT])?;
    let parent = types.intern(Kind::Tuple(fields))?;
    let mut query = NeedsDrop::scoped_bounded([].into_iter(), Arc::clone(&environment), LIMIT)?;
    let equal = Arc::new(environment.as_ref().clone());
    assert!(
        query
            .seed_scoped_copy(&types, &equal, &[(actual, true)], LIMIT)
            .is_err()
    );
    assert!(
        query
            .seed_scoped_copy(
                &types,
                &environment,
                &[(actual, true), (TypeId(u32::MAX), false)],
                LIMIT
            )
            .is_err()
    );
    assert!(query.scope.as_ref().ok_or("scope")?.states.is_empty());
    let bytes = query.retained_bytes();
    assert!(
        query
            .seed_scoped_copy(&types, &environment, &[(actual, true)], bytes)
            .is_err()
    );
    assert!(query.scope.as_ref().ok_or("scope")?.states.is_empty());
    query.seed_scoped_copy(
        &types,
        &environment,
        &[(actual, true), (mismatch, false)],
        LIMIT,
    )?;
    assert!(!query.demand_scoped_bounded(&types, &Definitions::default(), parent, LIMIT)?);
    assert!(query.demand_scoped_bounded(&types, &Definitions::default(), mismatch, LIMIT)?);
    assert!(
        query
            .seed_scoped_copy(&types, &environment, &[(mismatch, true)], LIMIT)
            .is_err()
    );
    assert!(query.demand_scoped_bounded(&types, &Definitions::default(), mismatch, LIMIT)?);
    Ok(())
}
