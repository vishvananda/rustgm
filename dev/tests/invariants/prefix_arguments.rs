//! Exact application keys and freshening witnesses never join unrelated calls.
use super::*;
use crate::{
    resolve::model::DefId,
    types::callable::Convention,
    types::{
        ItemId, Kind, TypeId,
        infer::{Domain, arguments::Argument, predicates::Predicate},
        parameters::predicates::Bound,
    },
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn signature(store: &mut Store) -> Result<TypeId> {
    let parameters = store.intern_list(&[])?;
    Ok(store.intern(Kind::Function {
        parameters,
        result: TypeId::UNIT,
        bound_regions: 0,
        unsafe_: false,
        convention: Convention::RUST,
    })?)
}
fn goal(context: &mut Context, store: &mut Store, input: TermId) -> Result<Predicate> {
    let mut substitution = crate::types::infer::Instantiation::new(context, store, &[])?;
    let value = context.instantiate_predicate(
        store,
        crate::types::parameters::predicates::Predicate {
            ty: TypeId::UNIT,
            bound: Bound::Sized,
        },
        &mut substitution,
    )?;
    Ok(value.map_arguments(|_| Ok(Argument::Type(input)))?)
}
fn append(
    context: &mut Context,
    store: &mut Store,
    calls: &mut super::super::calls::CallTypes,
    index: u32,
) -> Result<(TermId, TermId)> {
    let signature = signature(store)?;
    let signature = context.known(store, signature)?;
    let input = context.variable(Domain::Any)?;
    let term = context.function_item(ItemId(12), signature, &[Argument::Type(input)])?;
    let source = NodeId(index);
    calls.generic.push((source, DefId(12), term));
    calls
        .obligations
        .push((source, goal(context, store, input)?));
    Ok((term, input))
}
#[test]
fn applications_and_variable_aliases_keep_distinct_owners() -> Result {
    let mut store = Store::default();
    let mut context = Context::default();
    let mut calls = super::super::calls::CallTypes::default();
    let mut index = Inputs::default();
    let (first, a) = append(&mut context, &mut store, &mut calls, 1)?;
    let (second, b) = append(&mut context, &mut store, &mut calls, 2)?;
    assert_eq!(
        index.owner(&mut context, &store, &calls, &[], Some(first), NodeId(40))?,
        Some((NodeId(1), vec![0]))
    );
    let alias = context.variable(Domain::Any)?;
    context.equate(&store, alias, first)?;
    context.equate(&store, a, b)?;
    assert_eq!(
        index.owner(&mut context, &store, &calls, &[], Some(alias), NodeId(41))?,
        Some((NodeId(1), vec![]))
    );
    assert_eq!(
        index.owner(&mut context, &store, &calls, &[], Some(second), NodeId(42))?,
        Some((NodeId(2), vec![1]))
    );
    Ok(())
}
#[test]
fn freshening_requires_real_edge_and_shared_complete_arguments() -> Result {
    for witnessed in [false, true] {
        let mut store = Store::default();
        let mut context = Context::default();
        let mut calls = super::super::calls::CallTypes::default();
        let mut index = Inputs::default();
        let (first, input) = append(&mut context, &mut store, &mut calls, 1)?;
        let (other, _) = append(&mut context, &mut store, &mut calls, 2)?;
        let signature = context.child(&store, first, 0)?;
        let fresh = context.function_item(ItemId(12), signature, &[Argument::Type(input)])?;
        let edges = [super::super::PendingAdjustment {
            node: NodeId(1),
            actual: first,
            expected: fresh,
            kind: super::super::AdjustmentKind::Equality,
            reset: false,
        }];
        if witnessed {
            index.freshening(0);
        }
        let result = index.owner(&mut context, &store, &calls, &edges, Some(fresh), NodeId(3))?;
        assert_eq!(result.is_some(), witnessed);
        let mut forged = edges;
        forged[0].expected = other;
        let mut index = Inputs::default();
        index.freshening(0);
        assert_eq!(
            index.owner(
                &mut context,
                &store,
                &calls,
                &forged,
                Some(other),
                NodeId(4)
            )?,
            Some((NodeId(2), vec![1]))
        );
    }
    Ok(())
}
#[test]
fn repeated_prefix_lookups_visit_only_appended_inputs() -> Result {
    for width in [8, 32, 128] {
        let mut store = Store::default();
        let mut context = Context::default();
        let mut calls = super::super::calls::CallTypes::default();
        let mut index = Inputs::default();
        for ordinal in 0..width {
            let (term, _) = append(&mut context, &mut store, &mut calls, ordinal)?;
            let expected = Some((NodeId(ordinal), vec![ordinal as usize]));
            assert_eq!(
                index.owner(&mut context, &store, &calls, &[], Some(term), NodeId(999))?,
                expected
            );
            for _ in 0..4 {
                assert_eq!(
                    index.owner(&mut context, &store, &calls, &[], Some(term), NodeId(999))?,
                    Some((NodeId(ordinal), vec![]))
                );
            }
        }
        eprintln!("prefix width={width} steps={}", context.steps);
        assert!(context.steps <= 100 * width as usize);
        assert_eq!(
            index.data.as_ref().ok_or("missing index")?.origins.len(),
            width as usize
        );
    }
    Ok(())
}
#[test]
fn prefix_storage_preserves_compact_public_records() {
    eprintln!(
        "prefix layout Work={} Calls={} Inputs={} TypeId={} Kind={} Value={} NodeFacts={}",
        size_of::<super::super::Work>(),
        size_of::<super::super::calls::CallTypes>(),
        size_of::<Inputs>(),
        size_of::<TypeId>(),
        size_of::<Kind>(),
        size_of::<crate::body::ir::Value>(),
        size_of::<crate::sema::model::NodeFacts>()
    );
    assert_eq!(size_of::<Inputs>(), 8);
    assert_eq!(size_of::<TypeId>(), 4);
    assert_eq!(size_of::<Kind>(), 24);
    assert_eq!(size_of::<crate::body::ir::Value>(), 24);
    assert_eq!(size_of::<crate::sema::model::NodeFacts>(), 80);
}

#[test]
fn restored_application_seeds_only_later_appended_requirements() -> Result {
    let mut store = Store::default();
    let mut context = Context::default();
    let mut calls = super::super::calls::CallTypes::default();
    let (term, input) = append(&mut context, &mut store, &mut calls, 1)?;
    let mut index = Inputs::default();
    assert_eq!(
        index.owner(&mut context, &store, &calls, &[], Some(term), NodeId(7))?,
        Some((NodeId(1), vec![0]))
    );
    let retained = std::mem::take(&mut index);
    calls
        .obligations
        .push((NodeId(1), goal(&mut context, &mut store, input)?));
    index = retained;
    assert_eq!(
        index.owner(&mut context, &store, &calls, &[], Some(term), NodeId(7))?,
        Some((NodeId(1), vec![1]))
    );
    assert_eq!(
        index.owner(&mut context, &store, &calls, &[], Some(term), NodeId(7))?,
        Some((NodeId(1), vec![]))
    );
    Ok(())
}
