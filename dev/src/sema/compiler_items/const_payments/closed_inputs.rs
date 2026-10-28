//! Closed source profile data preserves every declared binder before payment.
use crate::{
    sema::model::Declarations,
    types::{
        Kind, ListId, Primitive, Store,
        parameters::{clauses::Clause, predicates::Predicate},
        substitute::Bound,
    },
};
pub(super) fn closed_arguments(store: &Store, parameters: ListId) -> Result<bool, &'static str> {
    let parameters = store
        .list(parameters)
        .ok_or("source const signature has invalid parameters")?;
    for &parameter in parameters {
        match store.kind(parameter) {
            Some(Kind::Primitive(primitive)) if primitive != Primitive::Str => {}
            Some(Kind::FunctionItem { signature, .. })
                if matches!(
                    store.kind(signature),
                    Some(Kind::Function {
                        bound_regions: 0,
                        unsafe_: false,
                        convention: crate::types::callable::Convention::RUST,
                        ..
                    })
                ) => {}
            _ => return Ok(false),
        }
    }
    Ok(true)
}
/// Reading a binder does not remove it; only fresh payment opens its body.
pub(super) fn closed_clause(store: &Store, clause: Clause) -> Result<Predicate, &'static str> {
    match clause {
        Clause::Predicate(predicate) => Ok(predicate),
        Clause::ForAll(id) => {
            let quantified = store
                .quantified(id)
                .ok_or("source const clause binder is missing")?;
            if quantified.bound_regions != 0 {
                return Err("nonempty quantified source const-call payment is not implemented");
            }
            Ok(quantified.predicate)
        }
    }
}
pub(super) fn open_closed_clause(
    facts: &mut Declarations,
    clause: Clause,
) -> Result<Predicate, &'static str> {
    let predicate = closed_clause(&facts.types, clause)?;
    if matches!(clause, Clause::Predicate(_)) {
        return Ok(predicate);
    }
    let mut fold = Bound::new(Vec::new());
    let result = fold.predicate(&mut facts.types, predicate);
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(fold.steps);
    result
}

/// Reserve the complete const materialization before copying clauses or binding
/// arguments. Dispatch and CTFE consume the same completed formal order.
pub(super) fn substitution(
    facts: &mut Declarations,
    instance: crate::types::instance::Instance,
    count: usize,
    budget: usize,
) -> Result<Bound, &'static str> {
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(2);
    let Some(Kind::FunctionItem { arguments, .. }) = facts.types.kind(instance.ty()) else {
        return Err("const requirement substitution lost its complete callable");
    };
    let arity = facts
        .types
        .arguments(arguments)
        .ok_or("invalid complete const-call arguments")?
        .len();
    facts.body_work.canonical_steps = facts
        .body_work
        .canonical_steps
        .saturating_add(4 * arity + 4);
    let temporary = arity
        .saturating_mul(8 * size_of::<crate::types::arguments::Argument>())
        .saturating_add(crate::support::retained::new_map::<
            crate::types::parameters::ParameterId,
            crate::types::arguments::Argument,
        >(arity));
    let storage = size_of::<super::Requirements>()
        .saturating_add(count.saturating_mul(size_of::<Clause>() * 2))
        .saturating_add(temporary)
        .saturating_add(
            size_of::<Bound>() + 3 * size_of::<Vec<crate::types::arguments::Argument>>(),
        );
    if storage > budget {
        return Err("source const-call requirements exceed retained storage budget");
    }
    // One full callable mapping serves every clause. An owner-only joint fold
    // cannot replace rigid parameters declared by the enclosing impl or trait.
    let fold = crate::sema::dispatch::callable_substitution(facts, instance)?;
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(fold.steps);
    Ok(fold)
}
