//! Bounded exact queries preflight live slots and share completed graph work.
use super::*;
use crate::types::{
    GenericArgs, Kind, Primitive, copy, opaque, parameters::environment::Environment,
};
use std::sync::Arc;
const LIMIT: usize = 1024 * 1024;

fn graph(types: &mut Store, width: usize, depth: usize) -> Result<TypeId, &'static str> {
    let mut ty = TypeId::primitive(Primitive::U8);
    for _ in 0..depth {
        let fields = types.intern_list(&vec![ty; width])?;
        ty = types.intern(Kind::Tuple(fields))?;
    }
    Ok(ty)
}

#[test]
fn bounded_constructor_preflights_without_consuming_registry() -> Result<(), &'static str> {
    let consumed = std::cell::Cell::new(0);
    let entries = || {
        (0..16).map(|id| {
            consumed.set(consumed.get() + 1);
            ItemId(id)
        })
    };
    for limit in [0, 1, 64] {
        assert!(NeedsDrop::concrete_bounded(entries(), limit).is_err());
        assert_eq!(consumed.get(), 0);
    }
    let query = NeedsDrop::concrete_bounded(entries(), LIMIT)?;
    assert_eq!(consumed.get(), 16);
    assert_eq!(query.custom.len(), 16);
    assert_eq!(query.steps, 17);
    assert!(query.retained_bytes() < LIMIT);
    assert!(query.work.is_empty());
    assert!(query.scope.as_deref().ok_or("scope")?.states.is_empty());
    Ok(())
}

#[test]
fn bounded_queries_reject_symbolic_or_dense_owners() -> Result<(), &'static str> {
    let types = Store::default();
    let definitions = Definitions::default();
    let mut dense = NeedsDrop::new([]);
    let environment = Arc::new(Environment::new(&types, &[])?);
    let mut symbolic = dense.scoped(environment);
    for query in [&mut dense, &mut symbolic] {
        assert_eq!(
            query.demand_concrete_bounded(&types, &definitions, TypeId::UNIT, LIMIT),
            Err("bounded destruction requires an exact sparse query")
        );
        assert_eq!(query.steps, 0);
    }
    Ok(())
}

#[test]
fn bounded_query_preflights_map_growth_and_keeps_prior_answers() -> Result<(), &'static str> {
    let types = Store::default();
    let definitions = Definitions::default();
    let mut query = NeedsDrop::concrete_bounded([].into_iter(), LIMIT)?;
    let mut primitives = Primitive::ALL.into_iter();
    loop {
        let primitive = primitives.next().ok_or("missing capacity control")?;
        assert!(!query.demand_concrete_bounded(
            &types,
            &definitions,
            TypeId::primitive(primitive),
            LIMIT
        )?);
        let scope = query.scope.as_deref().ok_or("scope")?;
        if scope.states.len() == scope.states.capacity() {
            break;
        }
    }
    let unseen = TypeId::primitive(primitives.next().ok_or("missing growth control")?);
    let scope = query.scope.as_deref().ok_or("scope")?;
    let before = (
        scope.states.len(),
        scope.states.capacity(),
        query.retained_bytes(),
    );
    assert_eq!(before.0, before.1);
    let work = query.steps;
    assert_eq!(
        query.demand_concrete_bounded(&types, &definitions, unseen, before.2),
        Err("destruction query storage budget exceeded")
    );
    let scope = query.scope.as_deref().ok_or("scope")?;
    assert_eq!(
        (
            scope.states.len(),
            scope.states.capacity(),
            query.retained_bytes()
        ),
        before
    );
    assert!(query.steps > work);
    assert!(
        query.steps - work >= before.0,
        "failed cleanup visits are charged"
    );
    assert!(query.work.is_empty());
    assert!(!query.demand_concrete_bounded(
        &types,
        &definitions,
        TypeId::primitive(Primitive::Bool),
        before.2
    )?);
    let consumed = query.steps;
    assert_eq!(
        query.demand_concrete_bounded(
            &types,
            &definitions,
            TypeId::primitive(Primitive::Bool),
            before.2 - 1
        ),
        Err("destruction query storage budget exceeded")
    );
    assert_eq!(query.steps, consumed + 1);
    assert_eq!(query.retained_bytes(), before.2);
    Ok(())
}

#[test]
fn bounded_query_preflights_work_growth_and_preserves_failure() -> Result<(), &'static str> {
    let mut types = Store::default();
    let definitions = Definitions::default();
    let mut query = NeedsDrop::concrete_bounded([].into_iter(), LIMIT)?;
    assert!(!query.demand_concrete_bounded(&types, &definitions, TypeId::UNIT, LIMIT)?);
    let before = query.retained_bytes();
    let work_capacity = query.work.capacity();
    let fields = types.intern_list(&[TypeId::UNIT; 128])?;
    let root = types.intern(Kind::Tuple(fields))?;
    let limit = before + 64;
    assert_eq!(
        query.demand_concrete_bounded(&types, &definitions, root, limit),
        Err("destruction query storage budget exceeded")
    );
    assert!(query.retained_bytes() <= limit);
    assert!(query.work.capacity() > work_capacity);
    assert!(query.work.is_empty());
    let consumed = query.steps;
    assert_eq!(
        query.demand_concrete_bounded(&types, &definitions, root, LIMIT),
        Err("destruction query storage budget exceeded")
    );
    assert_eq!(query.steps, consumed + 1);
    let mut fresh = NeedsDrop::concrete_bounded([].into_iter(), LIMIT)?;
    assert!(!fresh.demand_concrete_bounded(&types, &definitions, root, LIMIT)?);
    Ok(())
}

#[test]
fn bounded_query_shares_registry_and_dag_across_roots_and_noise() -> Result<(), &'static str> {
    let mut baseline = None;
    for noise in [0, 8192] {
        let mut types = Store::default();
        for length in 0..noise {
            types.intern(Kind::Array {
                element: TypeId::primitive(Primitive::Bool),
                length,
            })?;
        }
        let shared = graph(&mut types, 8, 32)?;
        let mut query = NeedsDrop::concrete_bounded((0..16).map(ItemId), LIMIT)?;
        let registry = Arc::as_ptr(&query.custom);
        let mut measured = Vec::new();
        for width in [8, 32, 128] {
            let fields = types.intern_list(&vec![shared; width])?;
            let root = types.intern(Kind::Tuple(fields))?;
            let start = query.steps;
            assert!(!query.demand_concrete_bounded(
                &types,
                &Definitions::default(),
                root,
                LIMIT
            )?);
            let visits = query.steps - start;
            if width != 8 {
                assert!(visits <= 4 * width + 4);
            }
            let bytes = query.retained_bytes();
            let start = query.steps;
            for _ in 0..128 {
                assert!(!query.demand_concrete_bounded(
                    &types,
                    &Definitions::default(),
                    root,
                    LIMIT
                )?);
            }
            assert_eq!(query.steps, start + 128);
            assert_eq!(query.retained_bytes(), bytes);
            assert_eq!(Arc::as_ptr(&query.custom), registry);
            let states = query.scope.as_deref().ok_or("scope")?.states.len();
            println!("noise={noise} width={width} steps={visits} states={states} bytes={bytes}");
            measured.push((visits, states, bytes));
        }
        if let Some(expected) = &baseline {
            assert_eq!(&measured, expected);
        } else {
            baseline = Some(measured);
        }
    }
    Ok(())
}

#[test]
fn bounded_pending_head_and_work_failures_never_become_false() -> Result<(), &'static str> {
    let mut types = Store::default();
    let definitions = Definitions::default();
    let id = types.declare_opaque(opaque::Declaration {
        trait_result: false,
        sized: true,
        copies: copy::Properties::default(),
    })?;
    let ty = types.intern(Kind::Opaque(id, GenericArgs::EMPTY))?;
    let mut pending = NeedsDrop::concrete_bounded([].into_iter(), LIMIT)?;
    assert!(
        pending
            .demand_concrete_bounded(&types, &definitions, ty, LIMIT)
            .is_err()
    );
    types.publish_storage_head(ty, TypeId::primitive(Primitive::U8))?;
    let consumed = pending.steps;
    assert!(
        pending
            .demand_concrete_bounded(&types, &definitions, ty, LIMIT)
            .is_err()
    );
    assert_eq!(pending.steps, consumed + 1);
    let mut ready = NeedsDrop::concrete_bounded([].into_iter(), LIMIT)?;
    assert!(!ready.demand_concrete_bounded(&types, &definitions, ty, LIMIT)?);
    ready.steps = 999_999;
    assert_eq!(
        ready.demand_concrete_bounded(&types, &definitions, TypeId::UNIT, LIMIT),
        Err("destruction query work budget exceeded")
    );
    let consumed = ready.steps;
    assert_eq!(
        ready.demand_concrete_bounded(&types, &definitions, TypeId::UNIT, LIMIT),
        Err("destruction query work budget exceeded")
    );
    assert!(ready.steps > consumed);
    Ok(())
}
