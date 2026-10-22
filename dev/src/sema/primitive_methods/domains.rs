//! Isolated numeric-domain evidence never admits rigid nominal or unconstrained inputs.
use super::{
    Candidate, Constructor, Context, Declarations, Descriptor, Form, Operands, Operation,
    Predicate, Primitive, Rhs, Scalars, TypeId, infer,
};
use crate::types::{Kind, RegionId};
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
    if let Some(candidate) = super::referential::candidate(facts, context, goal, right_term, kind)?
    {
        return Ok(candidate);
    }
    if let Some(candidate) = super::sequences::candidate(facts, context, goal, right_term, kind)? {
        return Ok(candidate);
    }
    let mut left = context.head(&facts.types, goal.subject())?;
    let mut right = context.head(&facts.types, right_term)?;
    if let Some(candidate) =
        super::ordering::candidate(facts, context, goal.subject(), right_term, kind)?
    {
        return Ok(candidate);
    }
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
                operands: Operands::Scalars(Scalars {
                    left,
                    right,
                    rhs: Rhs::Owned,
                }),
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

pub(super) fn validate_scalars(
    facts: &Declarations,
    descriptor: Descriptor,
    [subject, right]: [TypeId; 2],
    [receiver, physical_right]: [TypeId; 2],
    signature: TypeId,
) -> Result<(), &'static str> {
    let Some(crate::types::Kind::Function {
        result,
        bound_regions,
        ..
    }) = facts.types.kind(signature)
    else {
        return Err("primitive method has no callable signature");
    };
    let subject_type = descriptor.validate_receiver(facts, receiver, result, bound_regions)?;
    let scalar = descriptor.scalars()?;
    let expected = descriptor.operand_type(facts, scalar.right)?;
    if descriptor.form() == Form::Comparison
        && !matches!(facts.types.kind(physical_right),Some(Kind::Reference{referent,mutable:false,region}) if referent==right && region==RegionId::bound(0,1)?)
    {
        return Err("comparison RHS differs from independent shared borrow contract");
    }
    let actual = match scalar.rhs {
        Rhs::Owned => right,
        Rhs::SharedReference => match facts.types.kind(right) {
            Some(Kind::Reference {
                referent,
                mutable: false,
                ..
            }) => referent,
            _ => return Err("primitive forwarding method requires a shared RHS"),
        },
    };
    if actual != expected
        || subject != subject_type
        || !crate::sema::scalar::binary(descriptor.operator()?, scalar.left, scalar.right)
    {
        return Err("primitive method arguments differ from selected descriptor");
    }
    Ok(())
}
