//! Isolated numeric-domain evidence never admits rigid nominal or unconstrained inputs.
use super::{
    Candidate, Constructor, Context, Declarations, Descriptor, Form, Operation, Predicate,
    Primitive, Rhs, TypeId, infer,
};
pub(super) fn comparison_candidate(
    facts: &Declarations,
    context: &mut Context,
    goal: &Predicate,
    kind: crate::support::comparisons::Comparison,
    arguments: &[crate::types::infer::arguments::Argument],
) -> Result<Candidate, infer::Error> {
    use crate::support::comparisons::Comparison;
    let right_term = match (kind, arguments) {
        (
            Comparison::PartialEq | Comparison::PartialOrd,
            [crate::types::infer::arguments::Argument::Type(right)],
        ) => *right,
        (Comparison::Eq | Comparison::Ord, []) => goal.subject(),
        _ => return Err(infer::Error::InvalidIdentity),
    };
    let mut left = context.head(&facts.types, goal.subject())?;
    let mut right = context.head(&facts.types, right_term)?;
    if let (Some(Constructor::Primitive(primitive)), None) = (left, right) {
        infer_numeric(facts, context, right_term, primitive)?;
        right = context.head(&facts.types, right_term)?;
    } else if let (None, Some(Constructor::Primitive(primitive))) = (left, right) {
        infer_numeric(facts, context, goal.subject(), primitive)?;
        left = context.head(&facts.types, goal.subject())?;
    }
    if left.is_some_and(|head| !matches!(head, Constructor::Primitive(_)))
        || right.is_some_and(|head| !matches!(head, Constructor::Primitive(_)))
    {
        return Ok(Candidate::Unrelated);
    }
    let (Some(Constructor::Primitive(left)), Some(Constructor::Primitive(right))) = (left, right)
    else {
        return Ok(if left.is_none() || right.is_none() {
            Candidate::Deferred
        } else {
            Candidate::Unrelated
        });
    };
    Ok(
        if left == right
            && left != Primitive::Str
            && (!matches!(kind, Comparison::Eq | Comparison::Ord) || !left.is_float())
        {
            Candidate::Present(Descriptor {
                operation: Operation::Comparison(kind),
                form: Form::Comparison,
                left,
                right,
                rhs: Rhs::Owned,
            })
        } else {
            Candidate::Unrelated
        },
    )
}
// A numeric domain cannot denote a local nominal RHS. These equations belong
// to this isolated candidate, and ordinary competing evidence still causes ambiguity.
fn infer_numeric(
    facts: &Declarations,
    context: &mut Context,
    term: crate::types::infer::TermId,
    primitive: Primitive,
) -> Result<(), infer::Error> {
    let permitted = match context.unresolved_domain(term)? {
        Some(infer::Domain::Integer) => primitive.integer().is_some(),
        Some(infer::Domain::Float) => primitive.is_float(),
        _ => false,
    };
    if permitted {
        let known = context.known(&facts.types, TypeId::primitive(primitive))?;
        context.equate(&facts.types, term, known)?;
    }
    Ok(())
}
