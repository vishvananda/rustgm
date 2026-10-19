//! Immutable environments preserve local binders and every competing premise.
use super::{Abilities, Environment};
use crate::types::{
    GenericArgs, ItemId, Kind, Primitive, RegionId, Store, TypeId,
    arguments::{Argument, ConstKind},
    parameters::{
        Parameter, ParameterKind,
        clauses::{Clause, Quantified, QuantifiedId},
        predicates::{Bound, Predicate},
    },
    projection, substitute, traits,
};
const BYTE: TypeId = TypeId::primitive(Primitive::U8);

fn trait_bound(
    store: &mut Store,
    ty: TypeId,
    arguments: &[Argument],
) -> Result<Predicate, &'static str> {
    let arguments = store.intern_arguments(arguments)?;
    let application = store.intern_trait_application(traits::Application {
        item: ItemId(7),
        arguments,
    })?;
    Ok(Predicate {
        ty,
        bound: Bound::Trait(application),
    })
}
fn quantify(store: &mut Store, count: u32, predicate: Predicate) -> Result<Clause, &'static str> {
    Ok(Clause::ForAll(store.intern_quantified(Quantified {
        bound_regions: count,
        predicate,
    })?))
}
fn reference(
    store: &mut Store,
    region: RegionId,
    referent: TypeId,
) -> Result<TypeId, &'static str> {
    store.intern(Kind::Reference {
        mutable: false,
        region,
        referent,
    })
}

#[test]
fn mixed_clauses_keep_declaration_order_without_granting_flat_permissions()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let a = trait_bound(&mut store, BYTE, &[])?;
    let b = trait_bound(&mut store, TypeId::UNIT, &[])?;
    let qa = quantify(&mut store, 0, a)?;
    let qb = quantify(&mut store, 1, b)?;
    let qc = quantify(&mut store, 2, a)?;
    let base = Environment::new(&store, &[])?;
    assert!(base.traits.is_none());
    let ordinary = base.clone().with_predicates(&store, &[a, a, b])?;
    assert_eq!(
        ordinary.trait_clauses().collect::<Vec<_>>(),
        [Clause::Predicate(a), Clause::Predicate(b)]
    );
    let expected = [qa, Clause::Predicate(a), qb, qc, Clause::Predicate(b)];
    let mixed = base
        .clone()
        .with_clauses(&store, &expected)?
        .with_clauses(&store, &[qa, Clause::Predicate(a), qb])?;
    assert_eq!(mixed.trait_clauses().collect::<Vec<_>>(), expected);
    assert_eq!(base.trait_clauses().count(), 0);
    assert!(!mixed.has_only_parameters());
    assert!(!mixed.has_shape_assumptions());
    assert!(!mixed.requires_region_mapping());
    let quantified = base.with_clauses(&store, &[qa, qb, qc])?;
    let Bound::Trait(application) = a.bound else {
        return Err("missing test trait");
    };
    assert!(!quantified.assumes_trait(a.ty, application));
    assert!(!quantified.assumes_sized(a.ty));
    assert_eq!(quantified.assumed(a.ty), Abilities::default());
    assert_eq!(quantified.trait_clauses().collect::<Vec<_>>(), [qa, qb, qc]);
    Ok(())
}

#[test]
fn mapping_preserves_empty_and_nested_binders_and_opens_captures_in_source_order()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let local = RegionId::bound(0, 0)?;
    let captured = RegionId::bound(1, 1)?;
    let nested_local = reference(&mut store, RegionId::bound(1, 0)?, BYTE)?;
    let nested_capture = reference(&mut store, RegionId::bound(2, 1)?, BYTE)?;
    let parameters = store.intern_list(&[nested_local, nested_capture])?;
    let callable = store.intern(Kind::Function {
        parameters,
        result: BYTE,
        bound_regions: 1,
        unsafe_: false,
    })?;
    let value = trait_bound(
        &mut store,
        callable,
        &[Argument::Lifetime(local), Argument::Lifetime(captured)],
    )?;
    let clause = quantify(&mut store, 1, value)?;
    let ordinary = trait_bound(
        &mut store,
        BYTE,
        &[Argument::Lifetime(RegionId::bound(0, 0)?)],
    )?;
    let empty_value = trait_bound(
        &mut store,
        BYTE,
        &[Argument::Lifetime(RegionId::bound(1, 2)?)],
    )?;
    let empty = quantify(&mut store, 0, empty_value)?;
    let clauses = [clause, Clause::Predicate(ordinary), empty];
    let environment = Environment::new(&store, &[])?.with_clauses(&store, &clauses)?;
    assert!(environment.requires_region_mapping());
    assert_eq!(clause.escaping_binders(&store)?, 1);
    let mut fold = substitute::Bound::open_regions(7)?;
    let mapped = environment.substitute_regions(&mut store, &mut fold)?;
    assert_eq!(
        fold.opened_regions()
            .collect::<std::collections::HashMap<_, _>>(),
        [
            (RegionId::bound(0, 1)?, RegionId(7)),
            (RegionId::bound(0, 0)?, RegionId(8)),
            (RegionId::bound(0, 2)?, RegionId(9)),
        ]
        .into_iter()
        .collect()
    );
    let nested_capture = reference(&mut store, RegionId(7), BYTE)?;
    let parameters = store.intern_list(&[nested_local, nested_capture])?;
    let callable = store.intern(Kind::Function {
        parameters,
        result: BYTE,
        bound_regions: 1,
        unsafe_: false,
    })?;
    let value = trait_bound(
        &mut store,
        callable,
        &[Argument::Lifetime(local), Argument::Lifetime(RegionId(7))],
    )?;
    let expected = quantify(&mut store, 1, value)?;
    let ordinary = trait_bound(&mut store, BYTE, &[Argument::Lifetime(RegionId(8))])?;
    let empty_value = trait_bound(&mut store, BYTE, &[Argument::Lifetime(RegionId(9))])?;
    let empty = quantify(&mut store, 0, empty_value)?;
    assert_eq!(
        mapped.trait_clauses().collect::<Vec<_>>(),
        [expected, Clause::Predicate(ordinary), empty]
    );
    assert_eq!(environment.trait_clauses().collect::<Vec<_>>(), clauses);
    assert_eq!(expected.escaping_binders(&store)?, 0);
    assert!(mapped.requires_region_mapping());
    Ok(())
}

#[test]
fn local_only_clauses_skip_outer_mapping_but_free_and_captured_regions_do_not()
-> Result<(), &'static str> {
    let mut store = Store::default();
    for (region, count, mapping, escaping) in [
        (RegionId::bound(0, 0)?, 1, false, 0),
        (RegionId::bound(1, 0)?, 0, true, 1),
        (RegionId::bound(2, 0)?, 1, true, 2),
        (RegionId(7), 0, true, 0),
        (RegionId::STATIC, 0, true, 0),
    ] {
        let subject = reference(&mut store, region, BYTE)?;
        for (subject, args) in [
            (subject, Vec::new()),
            (BYTE, vec![Argument::Lifetime(region)]),
        ] {
            let predicate = trait_bound(&mut store, subject, &args)?;
            let clause = quantify(&mut store, count, predicate)?;
            let environment = Environment::new(&store, &[])?.with_clauses(&store, &[clause])?;
            assert_eq!(clause.has_free_regions(&store)?, mapping);
            assert_eq!(clause.escaping_binders(&store)?, escaping);
            assert_eq!(environment.requires_region_mapping(), mapping);
        }
    }
    Ok(())
}

#[test]
fn quantified_inputs_still_require_type_and_const_membership() -> Result<(), &'static str> {
    let mut store = Store::default();
    let ty_id = store.intern_parameter(Parameter {
        owner: ItemId(1),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let const_id = store.intern_parameter(Parameter {
        owner: ItemId(1),
        index: 1,
        kind: ParameterKind::Const { ty: BYTE },
    })?;
    let ty = store.intern(Kind::Parameter(ty_id))?;
    let constant = store.intern_const(ConstKind::Parameter(const_id))?;
    let value = trait_bound(
        &mut store,
        ty,
        &[
            Argument::Const(constant),
            Argument::Lifetime(RegionId::bound(0, 0)?),
        ],
    )?;
    let clause = quantify(&mut store, 1, value)?;
    let base = Environment::new(&store, &[])?;
    assert!(base.clone().with_clauses(&store, &[clause]).is_err());
    let typed = Environment::new(&store, &[(ty_id, Abilities::default())])?;
    assert!(typed.clone().with_clauses(&store, &[clause]).is_err());
    assert!(
        base.clone()
            .with_const_parameters(&store, &[const_id])?
            .with_clauses(&store, &[clause])
            .is_err()
    );
    let complete = typed
        .with_const_parameters(&store, &[const_id])?
        .with_clauses(&store, &[clause])?;
    assert_eq!(complete.trait_clauses().collect::<Vec<_>>(), [clause]);
    assert!(
        base.clone()
            .with_clauses(&store, &[Clause::ForAll(QuantifiedId(u32::MAX))])
            .is_err()
    );
    for bound in [
        Bound::Sized,
        Bound::Copy,
        Bound::Clone,
        Bound::ProjectionEqual(BYTE),
    ] {
        let concrete = quantify(&mut store, 0, Predicate { ty: BYTE, bound })?;
        assert_eq!(
            base.clone().with_clauses(&store, &[concrete]).is_err(),
            matches!(bound, Bound::ProjectionEqual(_))
        );
        let foreign = quantify(&mut store, 0, Predicate { ty, bound })?;
        assert!(base.clone().with_clauses(&store, &[foreign]).is_err());
    }
    Ok(())
}

#[test]
fn quantified_lifetime_membership_is_checked_before_the_separate_region_consumer()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let ty_id = store.intern_parameter(Parameter {
        owner: ItemId(1),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let const_id = store.intern_parameter(Parameter {
        owner: ItemId(1),
        index: 1,
        kind: ParameterKind::Const { ty: BYTE },
    })?;
    let ty = store.intern(Kind::Parameter(ty_id))?;
    let constant = store.intern_const(ConstKind::Parameter(const_id))?;
    let arguments = store.intern_arguments(&[Argument::Type(ty), Argument::Const(constant)])?;
    let subject = store.intern(Kind::Adt(ItemId(7), arguments))?;
    let clause = quantify(
        &mut store,
        1,
        Predicate {
            ty: subject,
            bound: Bound::Outlives(RegionId::bound(0, 0)?),
        },
    )?;
    assert!(clause.is_outlives(&store)?);
    assert!(
        Clause::ForAll(QuantifiedId(u32::MAX))
            .is_outlives(&store)
            .is_err()
    );
    for mask in 0..4 {
        let parameters = if mask & 1 == 0 {
            Vec::new()
        } else {
            vec![(ty_id, Abilities::default())]
        };
        let mut environment = Environment::new(&store, &parameters)?;
        if mask & 2 != 0 {
            environment = environment.with_const_parameters(&store, &[const_id])?;
        }
        let result = environment.with_clauses(&store, &[clause]);
        assert_eq!(result.is_ok(), mask == 3);
        if let Ok(environment) = result {
            assert_eq!(environment.trait_clauses().count(), 0);
            assert!(!environment.assumes_sized(subject));
            assert!(!environment.requires_region_mapping());
            assert_eq!(environment.assumed(subject), Abilities::default());
        }
    }
    Ok(())
}

#[test]
fn projected_buckets_include_quantified_competitors_before_and_after_mapping()
-> Result<(), &'static str> {
    let mut store = Store::default();
    store.declare_projection(
        ItemId(9),
        projection::Schema {
            trait_item: ItemId(8),
            sized: false,
            copies: crate::types::copy::Properties::default(),
        },
    )?;
    let parent = store.intern_trait_application(traits::Application {
        item: ItemId(8),
        arguments: GenericArgs::EMPTY,
    })?;
    let subject = store.intern(Kind::Projection(projection::Application {
        item: ItemId(9),
        subject: BYTE,
        trait_application: parent,
        arguments: GenericArgs::EMPTY,
    }))?;
    let ordinary = trait_bound(
        &mut store,
        subject,
        &[Argument::Lifetime(RegionId::bound(0, 0)?)],
    )?;
    let quantified = quantify(&mut store, 1, ordinary)?;
    let clauses = [Clause::Predicate(ordinary), quantified];
    let environment = Environment::new(&store, &[])?.with_clauses(&store, &clauses)?;
    assert_eq!(
        environment
            .projected_trait_candidates(ItemId(7), ItemId(9))
            .collect::<Vec<_>>(),
        clauses
    );
    assert_eq!(
        environment
            .projected_trait_candidates(ItemId(8), ItemId(9))
            .count(),
        0
    );
    let mut fold = substitute::Bound::new(vec![RegionId(5)]);
    let mapped = environment.substitute_regions(&mut store, &mut fold)?;
    let ordinary = trait_bound(&mut store, subject, &[Argument::Lifetime(RegionId(5))])?;
    assert_eq!(
        mapped
            .projected_trait_candidates(ItemId(7), ItemId(9))
            .collect::<Vec<_>>(),
        [Clause::Predicate(ordinary), quantified]
    );
    Ok(())
}

#[test]
fn distinct_environment_clauses_share_mapping_work_on_growing_captured_tails()
-> Result<(), &'static str> {
    let mut previous = None;
    for width in [64, 128, 256, 512] {
        let mut store = Store::default();
        let id = store.intern_parameter(Parameter {
            owner: ItemId(1),
            index: 0,
            kind: ParameterKind::Type { sized: true },
        })?;
        let ty = store.intern(Kind::Parameter(id))?;
        let mut tail = ty;
        let mut clauses = Vec::new();
        for _ in 0..width {
            tail = reference(&mut store, RegionId::bound(1, 0)?, tail)?;
            let value = trait_bound(
                &mut store,
                ty,
                &[
                    Argument::Type(tail),
                    Argument::Lifetime(RegionId::bound(0, 0)?),
                ],
            )?;
            clauses.push(quantify(&mut store, 1, value)?);
        }
        let environment = Environment::new(&store, &[(id, Abilities::default())])?
            .with_clauses(&store, &clauses)?;
        let before = store.len();
        let mut fold = substitute::Bound::new(vec![RegionId(7)]);
        let mapped = environment.substitute_regions(&mut store, &mut fold)?;
        assert_eq!(mapped.trait_clauses().count(), width);
        assert!(store.len() - before <= width + 1);
        assert!(fold.steps <= width * 12 + 32, "{width}: {}", fold.steps);
        if let Some(old) = previous {
            assert!(fold.steps <= 2 * old + 32);
        }
        previous = Some(fold.steps);
        for clause in mapped.trait_clauses() {
            assert_eq!(clause.escaping_binders(&store)?, 0);
            assert!(clause.has_free_regions(&store)?);
        }
        assert_eq!(environment.trait_clauses().collect::<Vec<_>>(), clauses);
    }
    Ok(())
}
