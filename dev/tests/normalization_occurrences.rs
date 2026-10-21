//! Projected occurrences retain scopes and readiness independently of output holes.
use rustgm::types::{
    ItemId, Primitive, Store, TypeId,
    infer::{
        Constructor, Context, Domain, Error,
        deferred::{Outcome, Relation},
    },
    lifetime_proof::{Obligations, ScopeId},
    projection::Schema,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const ITEM: ItemId = ItemId(9);

fn projection_store() -> Result<Store, &'static str> {
    let mut store = Store::default();
    store.declare_projection(
        ITEM,
        Schema {
            trait_item: ItemId(11),
            sized: true,
            copies: rustgm::types::copy::Properties::default(),
        },
    )?;
    Ok(store)
}

#[test]
fn output_holes_do_not_block_an_independent_projection_query() -> TestResult {
    let mut store = projection_store()?;
    let mut context = Context::default();
    let input = context.known(&store, TypeId::primitive(Primitive::U16))?;
    let projection = context.projected(&store, ITEM, input, &[], &[])?;
    let output = context.variable(Domain::Any)?;
    let alias = context.construct(Constructor::Tuple, &[output, projection])?;
    let roots = context.normalization_roots(&store, alias, ScopeId::ROOT, |_, _| {
        Err(Error::InvalidIdentity)
    })?;
    assert_eq!(roots, vec![(ScopeId::ROOT, projection)]);
    let watch = context.watch_resolution(&[roots[0].1], &[])?;
    assert_eq!(context.ready_resolution(&store)?, Some(watch));
    let Outcome::Pending(equation) =
        context.equate_deferred(&store, projection, output, Relation::Shape)?
    else {
        return Err("output equation must await the semantic result".into());
    };
    let byte = TypeId::primitive(Primitive::U8);
    context.apply_normalization(&mut store, projection, byte)?;
    assert_eq!(
        context.resume_equation(&store, equation)?,
        Outcome::Complete
    );
    assert_eq!(context.resolved(&mut store, output)?, byte);
    Ok(())
}

#[test]
fn nested_and_sibling_callables_keep_their_actual_scope_counts() -> TestResult {
    let store = projection_store()?;
    let mut context = Context::default();
    let input = context.known(&store, TypeId::primitive(Primitive::U16))?;
    let projection = context.projected(&store, ITEM, input, &[], &[])?;
    let unit = context.known(&store, TypeId::UNIT)?;
    let function = |count| Constructor::Function {
        bound_regions: count,
        parameters: 1,
        unsafe_: false,
        convention: rustgm::types::callable::Convention::RUST,
    };
    let inner = context.construct(function(0), &[projection, unit])?;
    let outer = context.construct(function(1), &[inner, unit])?;
    let sibling = context.construct(function(2), &[projection, unit])?;
    let root = context.construct(Constructor::Tuple, &[outer, sibling])?;
    let mut proof = Obligations::new(1);
    let ambient = proof.scope(ScopeId::ROOT, 3)?;
    let mut scopes = Vec::new();
    let roots = context.normalization_roots(&store, root, ambient, |parent, count| {
        let scope = proof
            .scope(parent, count)
            .map_err(|_| Error::InvalidIdentity)?;
        scopes.push((scope, parent, count));
        Ok(scope)
    })?;
    assert_eq!(scopes.len(), 3);
    assert_eq!((scopes[0].1, scopes[0].2), (ambient, 1));
    assert_eq!((scopes[1].1, scopes[1].2), (scopes[0].0, 0));
    assert_eq!((scopes[2].1, scopes[2].2), (ambient, 2));
    assert_eq!(
        roots,
        vec![(scopes[1].0, projection), (scopes[2].0, projection)]
    );
    Ok(())
}

#[test]
fn projection_inputs_stay_in_their_query_and_shared_ordinary_graphs_are_visited_once() -> TestResult
{
    let store = projection_store()?;
    let mut context = Context::default();
    let input = context.known(&store, TypeId::primitive(Primitive::U16))?;
    let inner = context.projected(&store, ITEM, input, &[], &[])?;
    let mut outer = inner;
    for _ in 0..512 {
        outer = context.projected(&store, ITEM, outer, &[], &[])?;
    }
    let before = context.steps;
    let roots = context.normalization_roots(&store, outer, ScopeId::ROOT, |_, _| {
        Err(Error::InvalidIdentity)
    })?;
    assert_eq!(roots, vec![(ScopeId::ROOT, outer)]);
    assert!(context.steps - before < 16);
    let mut diamond = context.construct(Constructor::Tuple, &[outer, inner])?;
    for _ in 0..512 {
        diamond = context.construct(Constructor::Tuple, &[diamond, diamond])?;
    }
    let before = context.steps;
    let roots = context.normalization_roots(&store, diamond, ScopeId::ROOT, |_, _| {
        Err(Error::InvalidIdentity)
    })?;
    assert_eq!(roots, vec![(ScopeId::ROOT, outer), (ScopeId::ROOT, inner)]);
    assert!(context.steps - before < 8192);
    Ok(())
}
