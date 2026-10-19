//! Completed structure may cross an isolated query; proof and unresolved inputs may not.
use super::*;
use crate::types::{
    Kind, Primitive, RegionId,
    infer::{Domain, arguments::Argument as Input, refinement::Refinement},
    substitute::Bound,
};

#[test]
fn isolated_demands_share_wide_region_free_inputs_without_early_interning() -> Result<(), Error> {
    for width in [16, 64, 256, 1024] {
        let mut store = Store::default();
        let mut caller = Context::default();
        let byte = caller.known(&store, TypeId::primitive(Primitive::U8))?;
        let wide = caller.construct(Constructor::Tuple, &vec![byte; width])?;
        let before = store.len();
        let (mut query, roots) = Refinement::new(&store, &mut caller, &[Input::Type(wide)])?;
        assert_eq!(store.len(), before);
        assert!(caller.settled_type(wide).is_none());
        let Input::Type(copied) = roots[0] else {
            return Err(Error::InvalidIdentity);
        };
        let before = query.context.steps;
        let mapped = query
            .context
            .map_regions(&mut store, copied, &mut Bound::universals(10))?;
        assert_eq!(mapped, copied);
        assert_eq!(query.context.steps - before, 1);
        let expected = query.context.resolved(&mut store, mapped)?;
        assert!(!query.publish(&mut store, &mut caller, &mut Bound::new(Vec::new()))?);
        assert_eq!(caller.settled_type(wide), Some(expected));
        assert!(caller.region_equalities.is_empty());
        let before = caller.steps;
        for _ in 0..width {
            let (mut query, roots) = Refinement::new(&store, &mut caller, &[Input::Type(wide)])?;
            let Input::Type(copied) = roots[0] else {
                return Err(Error::InvalidIdentity);
            };
            assert_eq!(query.context.terms.len(), 1);
            assert_eq!(query.context.resolved(&mut store, copied)?, expected);
            let committed = query.commit(&mut store, &mut caller, &mut Bound::new(Vec::new()))?;
            assert!(!committed.changed);
            assert!(committed.equalities.is_empty());
        }
        assert_eq!(caller.steps - before, 3 * width);
    }
    Ok(())
}

#[test]
fn unresolved_constants_and_types_do_not_receive_completed_certificates() -> Result<(), Error> {
    let store = Store::default();
    let mut caller = Context::default();
    let byte = caller.known(&store, TypeId::primitive(Primitive::U8))?;
    let hole = caller.variable(Domain::Any)?;
    let tuple = caller.construct(Constructor::Tuple, &[byte, hole])?;
    let length = caller.const_variable(&store, TypeId::primitive(Primitive::Usize))?;
    let array = caller.construct(Constructor::ArrayInfer(length), &[byte])?;
    let nominal = caller.nominal(crate::types::ItemId(0), &[Input::Const(length)])?;
    for root in [tuple, array, nominal] {
        let (mut query, roots) = Refinement::new(&store, &mut caller, &[Input::Type(root)])?;
        assert!(caller.completed(&store, root)?.is_none());
        let Input::Type(copied) = roots[0] else {
            return Err(Error::InvalidIdentity);
        };
        assert!(query.context.completed(&store, copied)?.is_none());
        assert!(query.context.unresolved_domain(copied)?.is_none());
    }
    Ok(())
}

#[test]
fn region_mapping_never_publishes_a_private_lifetime_to_the_caller() -> Result<(), Error> {
    let mut store = Store::default();
    let mut caller = Context::default();
    let byte = caller.known(&store, TypeId::primitive(Primitive::U8))?;
    let local = RegionId::bound(0, 0).map_err(|_| Error::InvalidIdentity)?;
    let reference = caller.construct(
        Constructor::Reference {
            mutable: false,
            region: local,
        },
        &[byte],
    )?;
    let (mut query, roots) = Refinement::new(&store, &mut caller, &[Input::Type(reference)])?;
    let Input::Type(copied) = roots[0] else {
        return Err(Error::InvalidIdentity);
    };
    let mapped = query
        .context
        .map_regions(&mut store, copied, &mut Bound::universals(10))?;
    assert_ne!(mapped, copied);
    let selected = query.context.resolved(&mut store, mapped)?;
    assert!(
        matches!(store.kind(selected), Some(Kind::Reference { region, .. }) if region != local)
    );
    let committed = query.commit(&mut store, &mut caller, &mut Bound::new(Vec::new()))?;
    assert!(!committed.changed);
    assert!(committed.equalities.is_empty());
    assert!(caller.settled_type(reference).is_none());
    let original = caller.resolved(&mut store, reference)?;
    assert!(
        matches!(store.kind(original), Some(Kind::Reference { region, .. }) if region == local)
    );
    Ok(())
}

#[test]
fn failed_reconciliation_does_not_publish_query_identities() -> Result<(), Error> {
    let mut store = Store::default();
    let mut caller = Context::default();
    let byte = caller.known(&store, TypeId::primitive(Primitive::U8))?;
    let wide = caller.construct(Constructor::Tuple, &[byte; 64])?;
    let hole = caller.variable(Domain::Any)?;
    let (mut query, roots) =
        Refinement::new(&store, &mut caller, &[Input::Type(wide), Input::Type(hole)])?;
    let [Input::Type(copied), Input::Type(query_hole)] = roots[..] else {
        return Err(Error::InvalidIdentity);
    };
    query.context.resolved(&mut store, copied)?;
    let boolean = query
        .context
        .known(&store, TypeId::primitive(Primitive::Bool))?;
    query.context.equate(&store, query_hole, boolean)?;
    caller.equate(&store, hole, byte)?;
    assert_eq!(
        query.publish(&mut store, &mut caller, &mut Bound::new(Vec::new())),
        Err(Error::Mismatch)
    );
    assert!(caller.settled_type(wide).is_none());
    assert_eq!(
        caller.resolved(&mut store, hole)?,
        TypeId::primitive(Primitive::U8)
    );
    Ok(())
}

#[test]
fn a_shape_fold_still_erases_unused_callable_binders() -> Result<(), Error> {
    let mut store = Store::default();
    let parameters = store.intern_list(&[]).map_err(|_| Error::InvalidIdentity)?;
    let callable = store
        .intern(Kind::Function {
            bound_regions: 1,
            parameters,
            result: TypeId::primitive(Primitive::U8),
            unsafe_: false,
        })
        .map_err(|_| Error::InvalidIdentity)?;
    let mut context = Context::default();
    let root = context.known(&store, callable)?;
    let mapped = context.map_regions(&mut store, root, &mut Bound::shape())?;
    let result = context.resolved(&mut store, mapped)?;
    assert!(matches!(
        store.kind(result),
        Some(Kind::Function {
            bound_regions: 0,
            ..
        })
    ));
    Ok(())
}
