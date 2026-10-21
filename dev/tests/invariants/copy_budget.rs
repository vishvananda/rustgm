//! Sparse closed Copy queries bound actual touched graphs, independent of the store.
use super::{Copies, State};
use crate::types::{Kind, Primitive, Store, TypeId, aggregate::Definitions};
use std::collections::HashMap;
fn diamond(store: &mut Store, depth: usize) -> Result<TypeId, &'static str> {
    let mut ty = TypeId::primitive(Primitive::U32);
    for _ in 0..depth {
        let fields = store.intern_list(&[ty, ty])?;
        ty = store.intern(Kind::Tuple(fields))?;
    }
    Ok(ty)
}
#[test]
fn sparse_copy_ignores_unrelated_types_and_reuses_completed_roots() -> Result<(), &'static str> {
    let mut store = Store::default();
    let root = diamond(&mut store, 32)?;
    let definitions = Definitions::default();
    let selected = HashMap::new();
    let mut first = Copies::bounded(1024);
    assert!(first.demand(&store, &definitions, &selected, root)?.copy);
    let expected = first.steps;
    for length in 0..10_000 {
        store.intern(Kind::Array {
            element: TypeId::primitive(Primitive::U8),
            length,
        })?;
    }
    let mut second = Copies::bounded(1024);
    assert!(second.demand(&store, &definitions, &selected, root)?.copy);
    assert_eq!(expected, second.steps);
    assert!(second.demand(&store, &definitions, &selected, root)?.copy);
    assert_eq!(expected, second.steps);
    assert_eq!(second.scope.as_ref().ok_or("scope")?.states.len(), 33);
    Ok(())
}
#[test]
fn bounded_copy_handles_deep_shared_diamonds_without_recursion() -> Result<(), &'static str> {
    let mut store = Store::default();
    let root = diamond(&mut store, 4096)?;
    let mut query = Copies::bounded(32_768);
    assert!(
        query
            .demand(&store, &Definitions::default(), &HashMap::new(), root)?
            .copy
    );
    assert!(query.steps <= 4097 * 7);
    assert_eq!(query.scope.as_ref().ok_or("scope")?.states.len(), 4097);
    Ok(())
}
#[test]
fn exhausted_copy_clears_pending_and_active_entries() -> Result<(), &'static str> {
    let mut store = Store::default();
    let root = diamond(&mut store, 128)?;
    let mut query = Copies::bounded(16);
    assert_eq!(
        query.demand(&store, &Definitions::default(), &HashMap::new(), root),
        Err("Copy query work budget exceeded")
    );
    assert!(query.work.is_empty());
    assert!(
        query
            .scope
            .as_ref()
            .ok_or("scope")?
            .states
            .values()
            .all(|state| !matches!(state, State::Active))
    );
    Ok(())
}
#[test]
fn failed_dependency_does_not_poison_independent_copy_queries() -> Result<(), &'static str> {
    use crate::types::{
        ItemId,
        parameters::{Parameter, ParameterKind},
    };
    let mut store = Store::default();
    let parameter = store.intern_parameter(Parameter {
        owner: ItemId(0),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let parameter = store.intern(Kind::Parameter(parameter))?;
    let fields = store.intern_list(&[parameter])?;
    let root = store.intern(Kind::Tuple(fields))?;
    let mut query = Copies::bounded(128);
    let definitions = Definitions::default();
    let selected = HashMap::new();
    assert_eq!(
        query.demand(&store, &definitions, &selected, root),
        Err("generic Copy checking requires a parameter environment")
    );
    assert!(query.work.is_empty());
    assert!(
        query
            .demand(
                &store,
                &definitions,
                &selected,
                TypeId::primitive(Primitive::U8)
            )?
            .copy
    );
    Ok(())
}
