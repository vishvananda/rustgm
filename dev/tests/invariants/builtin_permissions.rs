//! Quantified builtin permissions preserve whole subjects and each caller's scope.
#[path = "builtin_permissions/equalities.rs"]
mod equalities;
use super::{Abilities, Environment};
use crate::types::{
    ItemId, Kind, RegionId, Store, TypeId,
    aggregate::Definitions,
    copy::Copies,
    destruction::{NeedsDrop, regions::Regions},
    parameters::{
        Parameter, ParameterId, ParameterKind,
        clauses::{Clause, Quantified},
        predicates::{Bound, Predicate},
        shape::Shapes,
    },
    substitute,
};
use std::{collections::HashMap, sync::Arc};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn parameter(types: &mut Store) -> Result<(ParameterId, TypeId)> {
    let id = types.intern_parameter(Parameter {
        owner: ItemId(3),
        index: 0,
        kind: ParameterKind::Type { sized: false },
    })?;
    Ok((id, types.intern(Kind::Parameter(id))?))
}
fn clause(types: &mut Store, ty: TypeId, count: u32, bound: Bound) -> Result<Clause> {
    Ok(Clause::ForAll(types.intern_quantified(Quantified {
        bound_regions: count,
        predicate: Predicate { ty, bound },
    })?))
}
fn reference(types: &mut Store, region: RegionId, referent: TypeId) -> Result<TypeId> {
    Ok(types.intern(Kind::Reference {
        mutable: false,
        region,
        referent,
    })?)
}
fn tuple(types: &mut Store, children: &[TypeId]) -> Result<TypeId> {
    let list = types.intern_list(children)?;
    Ok(types.intern(Kind::Tuple(list))?)
}
fn pair(types: &mut Store, a: RegionId, b: RegionId, ty: TypeId) -> Result<TypeId> {
    let a = reference(types, a, TypeId::UNIT)?;
    let b = reference(types, b, TypeId::UNIT)?;
    tuple(types, &[a, b, ty])
}
fn environment(
    types: &Store,
    parameter: ParameterId,
    clauses: &[Clause],
) -> Result<Arc<Environment>> {
    Ok(Arc::new(
        Environment::new(types, &[(parameter, Abilities::default())])?
            .with_clauses(types, clauses)?,
    ))
}

#[test]
fn builtin_parameter_permissions_reach_copy_shape_and_drop_consumers() -> Result {
    let mut types = Store::default();
    let (id, ty) = parameter(&mut types)?;
    let definitions = Definitions::default();
    for bound in [Bound::Copy, Bound::Clone, Bound::Sized] {
        for count in [0, 1, 8] {
            let clause = clause(&mut types, ty, count, bound)?;
            let environment = environment(&types, id, &[clause])?;
            assert_eq!(environment.assumed(ty), Abilities::default());
            assert_eq!(environment.trait_clauses().count(), 0);
            assert!(environment.has_shape_assumptions());
            assert!(!environment.has_only_parameters());
            let mut copies = Copies::in_environment(Arc::clone(&environment));
            let properties = copies.demand(&types, &definitions, &HashMap::new(), ty)?;
            assert_eq!(properties.copy, bound == Bound::Copy);
            assert_eq!(properties.clone_, bound != Bound::Sized);
            let mut shapes = Shapes::new(Arc::clone(&environment));
            assert!(shapes.demand(&types, ty)?.sized);
            let mut drop = NeedsDrop::in_environment([], Arc::clone(&environment));
            assert_eq!(drop.demand(&types, &definitions, ty)?, bound != Bound::Copy);
            let mut regions = Regions::new([])?.scoped(environment);
            let parts = regions.symbolic(&types, &definitions, ty)?;
            assert_eq!(parts.parameters.is_empty(), bound == Bound::Copy);
        }
    }
    Ok(())
}

#[test]
fn builtin_matching_preserves_full_subjects_and_correlated_slots() -> Result {
    let mut types = Store::default();
    let (id, ty) = parameter(&mut types)?;
    let local = RegionId::bound(0, 0)?;
    let pattern = pair(&mut types, local, local, ty)?;
    let clause = clause(&mut types, pattern, 1, Bound::Copy)?;
    let environment = environment(&types, id, &[clause])?;
    let mut query = None;
    for (a, b, expected) in [(7, 7, true), (7, 8, false), (8, 8, true), (8, 7, false)] {
        let actual = pair(&mut types, RegionId(a), RegionId(b), ty)?;
        assert_eq!(
            environment.assumed_with(&types, actual, &mut query)?.copy,
            expected
        );
    }
    assert!(!environment.assumed_with(&types, ty, &mut query)?.copy);
    assert!(!environment.requires_region_mapping());
    let outside = Arc::new(Environment::new(&types, &[(id, Abilities::default())])?);
    assert!(outside.assumed_with(&types, pattern, &mut query).is_err());
    assert!(!outside.assumed_with(&types, pattern, &mut None)?.copy);
    Ok(())
}

#[test]
fn builtin_matching_relocates_captures_but_never_opens_clause_locals() -> Result {
    let mut types = Store::default();
    let (id, ty) = parameter(&mut types)?;
    let pattern = pair(
        &mut types,
        RegionId::bound(1, 0)?,
        RegionId::bound(0, 0)?,
        ty,
    )?;
    let clause = clause(&mut types, pattern, 1, Bound::Copy)?;
    let environment = environment(&types, id, &[clause])?;
    assert!(environment.requires_region_mapping());
    for depth in [0, 1, 3] {
        for slot in [0, 1] {
            let actual = pair(&mut types, RegionId::bound(depth, slot)?, RegionId(8), ty)?;
            assert_eq!(
                environment.matching_sized(&types, actual, depth, &mut None)?,
                slot == 0
            );
        }
    }
    let mut fold = substitute::Bound::open_regions(7)?;
    let opened = Arc::new(environment.substitute_regions(&mut types, &mut fold)?);
    assert_eq!(
        fold.opened_regions().collect::<Vec<_>>(),
        [(RegionId::bound(0, 0)?, RegionId(7))]
    );
    let actual = pair(&mut types, RegionId(7), RegionId(8), ty)?;
    assert!(opened.assumed_with(&types, actual, &mut None)?.copy);
    assert!(!environment.assumed_with(&types, actual, &mut None)?.copy);
    Ok(())
}

fn callable(types: &mut Store, input: RegionId, result: RegionId) -> Result<TypeId> {
    let local = reference(types, RegionId::bound(0, 0)?, TypeId::UNIT)?;
    let input = reference(types, input, TypeId::UNIT)?;
    let result = reference(types, result, TypeId::UNIT)?;
    let parameters = types.intern_list(&[local, input])?;
    Ok(types.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    })?)
}

#[test]
fn builtin_matching_does_not_capture_nested_callable_lifetimes() -> Result {
    let mut types = Store::default();
    let (id, ty) = parameter(&mut types)?;
    let region = RegionId::bound(1, 0)?;
    let function = callable(&mut types, region, region)?;
    let pattern = tuple(&mut types, &[function, ty])?;
    let clause = clause(&mut types, pattern, 1, Bound::Clone)?;
    let environment = environment(&types, id, &[clause])?;
    let mut query = None;
    for (input, result, expected) in [
        (RegionId::bound(1, 1)?, RegionId::bound(1, 1)?, true),
        (RegionId::bound(1, 1)?, RegionId::bound(1, 2)?, false),
        (RegionId::bound(0, 0)?, RegionId::bound(0, 0)?, false),
    ] {
        let function = callable(&mut types, input, result)?;
        let actual = tuple(&mut types, &[function, ty])?;
        let abilities = environment.assumed_with(&types, actual, &mut query)?;
        assert_eq!(abilities.clone_, expected);
        assert!(!abilities.copy);
    }
    Ok(())
}

#[test]
fn builtin_matching_shares_growing_prefixes_and_repeated_queries() -> Result {
    for width in [16_u16, 64, 256, 1024] {
        let mut types = Store::default();
        let mut pattern = TypeId::UNIT;
        let mut actual = TypeId::UNIT;
        let mut clauses = Vec::new();
        let mut roots = Vec::new();
        for slot in 0..width {
            pattern = reference(&mut types, RegionId::bound(0, slot)?, pattern)?;
            actual = reference(&mut types, RegionId(u32::from(slot) + 1), actual)?;
            clauses.push(clause(&mut types, pattern, u32::from(width), Bound::Copy)?);
            roots.push(actual);
        }
        let environment = Arc::new(Environment::new(&types, &[])?.with_clauses(&types, &clauses)?);
        let mut query = None;
        for &ty in &roots {
            assert!(environment.assumed_with(&types, ty, &mut query)?.copy);
        }
        let work = query.as_ref().ok_or("missing builtin query")?.steps();
        assert!(work < 128 * usize::from(width), "{width}: {work}");
        for ty in roots {
            assert!(environment.assumed_with(&types, ty, &mut query)?.copy);
        }
        let repeated = query.as_ref().ok_or("missing builtin query")?.steps() - work;
        assert_eq!(repeated, usize::from(width));
        eprintln!("builtin permissions width={width} first={work} repeated={repeated}");
    }
    Ok(())
}

#[test]
fn builtin_matching_deduplicates_unused_quantifiers_without_erasing_recorded_scopes() -> Result {
    for width in [16_u32, 128, 1024] {
        let mut types = Store::default();
        let local = RegionId::bound(0, 0)?;
        let pattern = pair(&mut types, local, local, TypeId::UNIT)?;
        let actual = pair(&mut types, RegionId(7), RegionId(8), TypeId::UNIT)?;
        let clauses: Vec<_> = (1..=width)
            .map(|count| clause(&mut types, pattern, count, Bound::Copy))
            .collect::<Result<_>>()?;
        let environment = Arc::new(Environment::new(&types, &[])?.with_clauses(&types, &clauses)?);
        assert_eq!(
            environment
                .builtins
                .as_ref()
                .ok_or("missing builtin clauses")?
                .clauses()
                .count(),
            width as usize
        );
        let mut query = None;
        for _ in 0..width {
            assert!(!environment.assumed_with(&types, actual, &mut query)?.copy);
        }
        assert!(query.as_ref().ok_or("missing builtin query")?.steps() < 8 * width as usize);
    }
    Ok(())
}
