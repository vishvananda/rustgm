//! Completed input graphs share structural work without hiding holes or projection updates.
use super::*;
use crate::types::{Primitive, RegionId, infer::Constructor};

#[test]
fn repeated_wide_inputs_resolve_and_import_in_constant_work() -> Result<(), Error> {
    for width in [16, 64, 256, 1024] {
        let mut store = Store::default();
        let mut context = Context::default();
        let byte = context.known(&store, TypeId::primitive(Primitive::U8))?;
        let wide = context.construct(Constructor::Tuple, &vec![byte; width])?;
        let expected = context.resolved(&mut store, wide)?;
        let application = store
            .intern_trait_application(crate::types::traits::Application {
                item: crate::types::ItemId(1),
                arguments: crate::types::GenericArgs::EMPTY,
            })
            .map_err(|_| Error::InvalidIdentity)?;
        let goal = crate::types::infer::predicates::Predicate::trait_application(
            wide,
            crate::types::ItemId(1),
            Vec::new(),
        );
        let selected = crate::types::parameters::predicates::Predicate {
            ty: expected,
            bound: crate::types::parameters::predicates::Bound::Trait(application),
        };
        let before = context.steps;
        for _ in 0..width {
            assert_eq!(context.resolved(&mut store, wide)?, expected);
            context.equate_proved_predicate(&mut store, &goal, selected)?;
            let mut isolated = Context::default();
            let copied = isolated.import(&store, &mut context, wide)?;
            assert_eq!(isolated.terms.len(), 1);
            assert_eq!(isolated.resolved(&mut store, copied)?, expected);
            assert_eq!(isolated.steps, 2);
        }
        assert_eq!(context.steps - before, 3 * width);
        assert_eq!(
            context
                .settled
                .as_ref()
                .map(|facts| facts.iter().flatten().count()),
            Some(1)
        );
    }
    Ok(())
}

#[test]
fn unresolved_inputs_remain_shared_after_completed_siblings_are_cached() -> Result<(), Error> {
    let mut store = Store::default();
    let mut context = Context::default();
    let byte = context.known(&store, TypeId::primitive(Primitive::U8))?;
    let sibling = context.construct(Constructor::Tuple, &[byte, byte])?;
    let hole = context.variable(Domain::Any)?;
    let root = context.construct(Constructor::Tuple, &[sibling, hole, hole])?;
    assert_eq!(context.resolved(&mut store, root), Err(Error::Unresolved));
    assert!(context.settled_type(sibling).is_some());
    assert!(context.settled_type(root).is_none());
    let mut isolated = Context::default();
    let copied = isolated.import(&store, &mut context, root)?;
    let left = isolated.child(&store, copied, 1)?;
    let right = isolated.child(&store, copied, 2)?;
    assert_eq!(left, right);
    let boolean = context.known(&store, TypeId::primitive(Primitive::Bool))?;
    context.equate(&store, hole, boolean)?;
    assert!(context.resolved(&mut store, root).is_ok());
    assert_eq!(
        isolated.resolved(&mut store, copied),
        Err(Error::Unresolved)
    );
    Ok(())
}

#[test]
fn cached_inputs_retain_lifetime_equations() -> Result<(), Error> {
    let mut store = Store::default();
    let mut context = Context::default();
    let byte = context.known(&store, TypeId::primitive(Primitive::U8))?;
    let reference = |context: &mut Context, region| {
        context.construct(
            Constructor::Reference {
                mutable: false,
                region,
            },
            &[byte],
        )
    };
    let left = reference(&mut context, RegionId(10))?;
    let right = reference(&mut context, RegionId(20))?;
    let original = context.resolved(&mut store, left)?;
    context.resolved(&mut store, right)?;
    context.equate(&store, left, right)?;
    assert!(
        context
            .region_equalities
            .contains(&(RegionId(10), RegionId(20)))
    );
    assert_eq!(context.resolved(&mut store, left)?, original);
    Ok(())
}

#[test]
fn expanding_a_canonical_input_retains_its_completed_identity() -> Result<(), Error> {
    let mut store = Store::default();
    let byte = TypeId::primitive(Primitive::U8);
    let fields = store
        .intern_list(&vec![byte; 256])
        .map_err(|_| Error::InvalidIdentity)?;
    let tuple = store
        .intern(crate::types::Kind::Tuple(fields))
        .map_err(|_| Error::InvalidIdentity)?;
    let mut context = Context::default();
    let root = context.known(&store, tuple)?;
    context.materialize(&store, root)?;
    assert_eq!(context.settled_type(root), Some(tuple));
    let mut isolated = Context::default();
    let copied = isolated.import(&store, &mut context, root)?;
    assert_eq!(isolated.steps, 1);
    assert_eq!(isolated.resolved(&mut store, copied)?, tuple);
    Ok(())
}
