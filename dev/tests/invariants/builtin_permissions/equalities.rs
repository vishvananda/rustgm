//! Assumed equivalence survives binder transport without authorizing local captures.
use super::*;
use crate::types::{relations::Constraints, variance::Variances};

fn equalities(
    types: &Store,
    pairs: &[(u16, u16)],
) -> Result<crate::types::relations::RegionEqualities> {
    let variances = Variances::default();
    let mut constraints = Constraints::new(types, &variances, 4, 10_000)?;
    for &(a, b) in pairs {
        constraints.assume(RegionId::bound(0, a)?, RegionId::bound(0, b)?)?;
    }
    Ok(constraints.assumed_equalities()?)
}

#[test]
fn ordinary_builtin_bounds_use_proven_identity_without_binding_caller_slots() -> Result {
    let mut types = Store::default();
    let (id, ty) = parameter(&mut types)?;
    let first = RegionId::bound(0, 0)?;
    let second = RegionId::bound(0, 1)?;
    let subject = pair(&mut types, first, first, ty)?;
    let actual = pair(&mut types, first, second, ty)?;
    let ordinary = Clause::Predicate(Predicate {
        ty: subject,
        bound: Bound::Copy,
    });
    let quantified = clause(&mut types, subject, 1, Bound::Sized)?;
    let original = environment(&types, id, &[ordinary, quantified])?;
    assert!(!original.assumed_with(&types, actual, &mut None)?.copy);
    let other = pair(&mut types, second, second, ty)?;
    assert!(!original.assumed_with(&types, other, &mut None)?.copy);
    assert!(original.matching_sized(&types, other, 0, &mut None)?);
    let classes = equalities(&types, &[(0, 1), (1, 0)])?;
    let proven = Arc::new(original.as_ref().clone().with_region_equalities(classes));
    assert!(proven.assumed_with(&types, actual, &mut None)?.copy);
    let mut fold = substitute::Bound::new(vec![RegionId(7), RegionId(8)]);
    let mapped = Arc::new(proven.substitute_regions(&mut types, &mut fold)?);
    let actual = pair(&mut types, RegionId(7), RegionId(8), ty)?;
    assert!(mapped.assumed_with(&types, actual, &mut None)?.copy);
    assert!(!original.assumed_with(&types, actual, &mut None)?.copy);
    Ok(())
}

#[test]
fn builtin_correlations_require_mutual_not_one_way_entailment() -> Result {
    let mut types = Store::default();
    let (id, ty) = parameter(&mut types)?;
    let local = RegionId::bound(0, 0)?;
    let pattern = pair(&mut types, local, local, ty)?;
    let actual = pair(&mut types, local, RegionId::bound(0, 1)?, ty)?;
    for (pairs, accepted) in [
        (&[(0, 1)][..], false),
        (&[(1, 0)][..], false),
        (&[(0, 2), (2, 1), (1, 0)][..], true),
    ] {
        for bound in [Bound::Copy, Bound::Clone, Bound::Sized] {
            let clause = clause(&mut types, pattern, 1, bound)?;
            let regions = equalities(&types, pairs)?;
            let environment = Arc::new(
                environment(&types, id, &[clause])?
                    .as_ref()
                    .clone()
                    .with_region_equalities(regions),
            );
            let permissions = environment.assumed_with(&types, actual, &mut None)?;
            assert_eq!(permissions.copy, accepted && bound == Bound::Copy);
            assert_eq!(permissions.clone_, accepted && bound != Bound::Sized);
            assert_eq!(
                environment.matching_sized(&types, actual, 0, &mut None)?,
                accepted
            );
            let definitions = Definitions::default();
            let mut drop = NeedsDrop::in_environment([], Arc::clone(&environment));
            assert_eq!(
                drop.demand(&types, &definitions, actual)?,
                !permissions.copy
            );
            let mut drop_regions = Regions::new([])?.scoped(environment);
            assert_eq!(
                drop_regions
                    .symbolic(&types, &definitions, actual)?
                    .parameters
                    .is_empty(),
                permissions.copy
            );
        }
    }
    Ok(())
}

#[test]
fn builtin_captured_equalities_respect_surrounding_and_nested_binders() -> Result {
    let mut types = Store::default();
    let (id, ty) = parameter(&mut types)?;
    let pattern = pair(
        &mut types,
        RegionId::bound(1, 0)?,
        RegionId::bound(0, 0)?,
        ty,
    )?;
    let clause = clause(&mut types, pattern, 1, Bound::Copy)?;
    let regions = equalities(&types, &[(0, 1), (1, 0)])?;
    let environment = Arc::new(
        environment(&types, id, &[clause])?
            .as_ref()
            .clone()
            .with_region_equalities(regions),
    );
    for depth in [0, 1, 3] {
        for slot in [0, 1, 2] {
            let actual = pair(&mut types, RegionId::bound(depth, slot)?, RegionId(8), ty)?;
            assert_eq!(
                environment.matching_sized(&types, actual, depth, &mut None)?,
                slot < 2
            );
        }
        if depth > 0 {
            let actual = pair(&mut types, RegionId::bound(depth - 1, 1)?, RegionId(8), ty)?;
            assert!(!environment.matching_sized(&types, actual, depth, &mut None)?);
        }
    }
    Ok(())
}

#[test]
fn builtin_equivalence_substitution_closes_colliding_classes_and_preserves_original() -> Result {
    let mut types = Store::default();
    let (id, ty) = parameter(&mut types)?;
    let local = RegionId::bound(0, 0)?;
    let pattern = pair(&mut types, local, local, ty)?;
    let clause = clause(&mut types, pattern, 1, Bound::Copy)?;
    let regions = equalities(&types, &[(0, 1), (1, 0), (2, 3), (3, 2)])?;
    let environment = environment(&types, id, &[clause])?
        .as_ref()
        .clone()
        .with_region_equalities(regions);
    let mut fold = substitute::Bound::new(vec![RegionId(7), RegionId(8), RegionId(8), RegionId(9)]);
    let mapped = Arc::new(environment.substitute_regions(&mut types, &mut fold)?);
    let original = Arc::new(environment);
    let actual = pair(&mut types, RegionId(7), RegionId(9), ty)?;
    assert!(mapped.assumed_with(&types, actual, &mut None)?.copy);
    assert!(!original.assumed_with(&types, actual, &mut None)?.copy);
    let unrelated = pair(&mut types, RegionId(7), RegionId(10), ty)?;
    assert!(!mapped.assumed_with(&types, unrelated, &mut None)?.copy);
    Ok(())
}

#[test]
fn builtin_equivalences_follow_explicit_capture_relocation() -> Result {
    let mut types = Store::default();
    let (id, ty) = parameter(&mut types)?;
    let local = RegionId::bound(0, 0)?;
    let pattern = pair(&mut types, local, local, ty)?;
    let clause = clause(&mut types, pattern, 1, Bound::Copy)?;
    let regions = equalities(&types, &[(0, 1), (1, 0)])?;
    let original = Arc::new(
        environment(&types, id, &[clause])?
            .as_ref()
            .clone()
            .with_region_equalities(regions),
    );
    let moved =
        Arc::new(original.substitute_regions(&mut types, &mut substitute::Bound::shift(2))?);
    for depth in [0, 1, 3] {
        let actual = pair(
            &mut types,
            RegionId::bound(depth + 2, 0)?,
            RegionId::bound(depth + 2, 1)?,
            ty,
        )?;
        assert!(moved.matching_sized(&types, actual, depth, &mut None)?);
        assert!(!original.matching_sized(&types, actual, depth, &mut None)?);
        let foreign = pair(
            &mut types,
            RegionId::bound(depth + 1, 0)?,
            RegionId::bound(depth + 1, 1)?,
            ty,
        )?;
        assert!(!moved.matching_sized(&types, foreign, depth, &mut None)?);
    }
    Ok(())
}

#[test]
fn builtin_equalities_do_not_identify_nested_callable_locals() -> Result {
    let mut types = Store::default();
    let (id, ty) = parameter(&mut types)?;
    let pattern_fn = callable(&mut types, RegionId::bound(1, 0)?, RegionId::bound(1, 0)?)?;
    let pattern = tuple(&mut types, &[pattern_fn, ty])?;
    let clause = clause(&mut types, pattern, 1, Bound::Copy)?;
    let regions = equalities(&types, &[(0, 1), (1, 0)])?;
    let environment = Arc::new(
        environment(&types, id, &[clause])?
            .as_ref()
            .clone()
            .with_region_equalities(regions),
    );
    for (a, b, accepted) in [
        (RegionId::bound(1, 0)?, RegionId::bound(1, 1)?, true),
        (RegionId::bound(0, 0)?, RegionId::bound(0, 0)?, false),
    ] {
        let function = callable(&mut types, a, b)?;
        let actual = tuple(&mut types, &[function, ty])?;
        assert_eq!(
            environment.assumed_with(&types, actual, &mut None)?.copy,
            accepted
        );
    }
    Ok(())
}

#[test]
fn exact_builtin_permissions_skip_matching_without_sharing_caller_authority() -> Result {
    let mut types = Store::default();
    let (id, ty) = parameter(&mut types)?;
    let region = RegionId::bound(0, 0)?;
    let subject = pair(&mut types, region, region, ty)?;
    let actual = pair(&mut types, RegionId(7), RegionId(7), ty)?;
    let ordinary = Clause::Predicate(Predicate {
        ty: actual,
        bound: Bound::Copy,
    });
    let quantified = clause(&mut types, subject, 1, Bound::Clone)?;
    let original = environment(&types, id, &[quantified])?;
    let extended = Arc::new(
        original
            .as_ref()
            .clone()
            .with_clauses(&types, &[ordinary])?,
    );
    let mut query = None;
    for _ in 0..1_024 {
        assert!(extended.assumed_with(&types, actual, &mut query)?.copy);
        assert!(extended.matching_sized(&types, actual, 0, &mut query)?);
    }
    assert!(
        query.is_none(),
        "exact permissions must not allocate a matcher"
    );
    let properties = original.assumed_with(&types, actual, &mut query)?;
    assert!(properties.clone_ && !properties.copy);
    assert!(extended.assumed_with(&types, actual, &mut query).is_err());
    assert!(
        extended
            .matching_sized(&types, actual, 0, &mut query)
            .is_err()
    );
    // An exact Clone fact is not enough to answer Copy. Quantified evidence
    // must still be consulted, and extending a clone must not alter its parent.
    let clone_only = environment(
        &types,
        id,
        &[Predicate {
            ty: actual,
            bound: Bound::Clone,
        }
        .into()],
    )?;
    let universal_copy = clause(&mut types, subject, 1, Bound::Copy)?;
    let copied = Arc::new(
        clone_only
            .as_ref()
            .clone()
            .with_clauses(&types, &[universal_copy])?,
    );
    assert!(copied.assumed_with(&types, actual, &mut None)?.copy);
    assert!(!clone_only.assumed_with(&types, actual, &mut None)?.copy);
    Ok(())
}
