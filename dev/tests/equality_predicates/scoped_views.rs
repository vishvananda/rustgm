//! Projection views distinguish identical term IDs beneath independent callable binders.
use super::{
    Argument, BYTE, Bound, Constructor, Context, Domain, Instantiation, Kind, Predicate, RegionId,
    Store, TestResult, TypeId, WORD, borrowed, infer, projection,
};
use rustgm::types::{infer::TermId, lifetime_proof::ScopeId};

fn goal(
    context: &mut Context,
    store: &mut Store,
    root: TermId,
) -> Result<infer::predicates::Predicate, infer::Error> {
    let mut substitution = Instantiation::new(context, store, &[])?;
    context
        .instantiate_predicate(
            store,
            Predicate {
                ty: BYTE,
                bound: Bound::Sized,
            },
            &mut substitution,
        )?
        .map_inputs(|_| Ok(root))
}

fn callable(
    context: &mut Context,
    argument: TermId,
    result: TermId,
    count: u32,
) -> Result<TermId, infer::Error> {
    context.construct(
        Constructor::Function {
            bound_regions: count,
            parameters: 1,
            unsafe_: false,
        },
        &[argument, result],
    )
}

fn signature(
    store: &mut Store,
    argument: TypeId,
    result: TypeId,
    count: u32,
) -> Result<TypeId, &'static str> {
    let parameters = store.intern_list(&[argument])?;
    store.intern(Kind::Function {
        bound_regions: count,
        parameters,
        result,
        unsafe_: false,
    })
}

#[test]
fn shared_projection_terms_have_distinct_root_and_sibling_replacements() -> TestResult {
    let mut store = Store::default();
    let local = RegionId::bound(0, 0)?;
    let projected = projection(&mut store, BYTE, &[Argument::Lifetime(local)])?;
    let mut context = Context::default();
    let term = context.known(&store, projected)?;
    let byte = context.known(&store, BYTE)?;
    let word = context.known(&store, WORD)?;
    let first = callable(&mut context, byte, term, 1)?;
    let second = callable(&mut context, word, term, 1)?;
    let root = context.construct(Constructor::Tuple, &[term, first, second])?;
    let goal = goal(&mut context, &mut store, root)?;
    let original = context.resolved_predicate(&mut store, &goal)?;
    let mut plan = context.projected_inputs(&store, &goal)?;
    let &[
        (root_scope, root_term),
        (first_scope, first_term),
        (second_scope, second_term),
    ] = plan.roots()
    else {
        return Err("expected root and two scoped projection occurrences".into());
    };
    assert_eq!((root_scope, root_term), (ScopeId::ROOT, term));
    assert_eq!((first_term, second_term), (term, term));
    assert_ne!(first_scope, second_scope);
    assert_ne!(first_scope, ScopeId::ROOT);
    for scope in [first_scope, second_scope] {
        assert_eq!(plan.binder(scope), Some((ScopeId::ROOT, 1)));
    }
    let reference = borrowed(&mut store, local, BYTE)?;
    plan.replace(ScopeId::ROOT, term, BYTE)?;
    plan.replace(first_scope, term, reference)?;
    assert!(matches!(
        plan.rewrite(&mut context, &store),
        Err(infer::Error::Normalization)
    ));
    assert!(plan.replace(first_scope, byte, WORD).is_err());
    assert!(plan.replace(first_scope, term, WORD).is_err());
    plan.replace(second_scope, term, WORD)?;
    let view = plan.rewrite(&mut context, &store)?;
    let first = signature(&mut store, BYTE, reference, 1)?;
    let second = signature(&mut store, WORD, WORD, 1)?;
    let elements = store.intern_list(&[BYTE, first, second])?;
    let expected = store.intern(Kind::Tuple(elements))?;
    assert_eq!(context.resolved(&mut store, view.subject())?, expected);
    assert_eq!(
        context.resolved_predicate(&mut store, plan.original())?,
        original
    );
    assert_eq!(context.resolved(&mut store, term)?, projected);
    Ok(())
}

#[test]
fn nested_empty_binders_preserve_captures_and_unrelated_input_holes() -> TestResult {
    let mut store = Store::default();
    let captured = RegionId::bound(1, 0)?;
    let projected = projection(&mut store, BYTE, &[Argument::Lifetime(captured)])?;
    let mut context = Context::default();
    let term = context.known(&store, projected)?;
    let byte = context.known(&store, BYTE)?;
    let hole = context.variable(Domain::Any)?;
    let inner = callable(&mut context, hole, term, 0)?;
    let outer = callable(&mut context, byte, inner, 1)?;
    let goal = goal(&mut context, &mut store, outer)?;
    let mut plan = context.projected_inputs(&store, &goal)?;
    let &[(inner_scope, found)] = plan.roots() else {
        return Err("expected one nested projection occurrence".into());
    };
    assert_eq!(found, term);
    let (outer_scope, count) = plan.binder(inner_scope).ok_or("missing inner binder")?;
    assert_eq!(count, 0);
    assert_eq!(plan.binder(outer_scope), Some((ScopeId::ROOT, 1)));
    let reference = borrowed(&mut store, captured, BYTE)?;
    plan.replace(inner_scope, term, reference)?;
    let view = plan.rewrite(&mut context, &store)?;
    assert_eq!(
        context.resolved(&mut store, hole),
        Err(infer::Error::Unresolved)
    );
    let word = context.known(&store, WORD)?;
    context.equate(&store, hole, word)?;
    let inner = signature(&mut store, WORD, reference, 0)?;
    let expected = signature(&mut store, BYTE, inner, 1)?;
    assert_eq!(context.resolved(&mut store, view.subject())?, expected);
    let original_inner = signature(&mut store, WORD, projected, 0)?;
    let original = signature(&mut store, BYTE, original_inner, 1)?;
    assert_eq!(
        context.resolved(&mut store, plan.original().subject())?,
        original
    );
    Ok(())
}

#[test]
fn a_new_projection_in_a_shared_hole_requires_a_new_plan() -> TestResult {
    let mut store = Store::default();
    let projected = projection(&mut store, BYTE, &[])?;
    let later = projection(&mut store, WORD, &[])?;
    let mut context = Context::default();
    let term = context.known(&store, projected)?;
    let hole = context.variable(Domain::Any)?;
    let root = context.construct(Constructor::Tuple, &[hole, term])?;
    let goal = goal(&mut context, &mut store, root)?;
    let mut plan = context.projected_inputs(&store, &goal)?;
    assert_eq!(plan.roots(), &[(ScopeId::ROOT, term)]);
    plan.replace(ScopeId::ROOT, term, WORD)?;
    let introduced = context.known(&store, later)?;
    context.equate(&store, hole, introduced)?;
    assert!(matches!(
        plan.rewrite(&mut context, &store),
        Err(infer::Error::Normalization)
    ));
    let fresh = context.projected_inputs(&store, &goal)?;
    assert_eq!(fresh.roots().len(), 2);
    Ok(())
}

#[test]
fn shared_callable_input_dags_keep_linear_rewriting_work() -> TestResult {
    for width in [16, 64, 256, 1024] {
        let mut store = Store::default();
        let projected = projection(&mut store, BYTE, &[])?;
        let mut context = Context::default();
        let term = context.known(&store, projected)?;
        let byte = context.known(&store, BYTE)?;
        let repeated = context.construct(Constructor::Tuple, &vec![term; width])?;
        let function = callable(&mut context, byte, repeated, 1)?;
        let root = context.construct(Constructor::Tuple, &vec![function; width])?;
        let goal = goal(&mut context, &mut store, root)?;
        let before = context.steps;
        let mut plan = context.projected_inputs(&store, &goal)?;
        let &[(scope, found)] = plan.roots() else {
            return Err("expected one shared projection occurrence".into());
        };
        assert_eq!(found, term);
        assert_eq!(plan.binder(scope), Some((ScopeId::ROOT, 1)));
        plan.replace(scope, term, WORD)?;
        let view = plan.rewrite(&mut context, &store)?;
        assert!(
            context.steps - before < width * 40,
            "width={width}: {}",
            context.steps - before
        );
        let elements = store.intern_list(&vec![WORD; width])?;
        let result = store.intern(Kind::Tuple(elements))?;
        let function = signature(&mut store, BYTE, result, 1)?;
        let elements = store.intern_list(&vec![function; width])?;
        let expected = store.intern(Kind::Tuple(elements))?;
        assert_eq!(context.resolved(&mut store, view.subject())?, expected);
        assert_eq!(context.resolved(&mut store, term)?, projected);
    }
    Ok(())
}
