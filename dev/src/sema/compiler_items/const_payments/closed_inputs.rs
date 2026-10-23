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
    let [function, scalar] = parameters else {
        return Ok(parameters.is_empty());
    };
    if *scalar != crate::types::TypeId::primitive(Primitive::U8) {
        return Ok(false);
    }
    let Some(Kind::FunctionItem { signature, .. }) = store.kind(*function) else {
        return Ok(false);
    };
    let Some(Kind::Function {
        parameters,
        bound_regions: 0,
        result,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    }) = store.kind(signature)
    else {
        return Ok(false);
    };
    Ok(result == *scalar && store.list(parameters) == Some(&[*scalar][..]))
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
