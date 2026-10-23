//! Structural eligibility creates all children; it is never a selected proof.
use crate::{
    sema::model::Declarations,
    types::infer::{Constructor, Context, Error, TermId, predicates::Predicate},
};
pub(super) enum Outcome {
    Ready,
    Absent,
    Deferred,
    Unsupported(&'static str),
}

pub(super) fn requirements(
    facts: &mut Declarations,
    context: &mut Context,
    source: TermId,
    target: TermId,
    goals: &mut Vec<Predicate>,
) -> Result<Outcome, Error> {
    context.account_work(1)?;
    let heads = (
        context.head(&facts.types, source)?,
        context.head(&facts.types, target)?,
    );
    match heads {
        (
            Some(Constructor::Array(_) | Constructor::ArrayTerm(_) | Constructor::ArrayInfer(_)),
            Some(Constructor::Slice),
        ) => {
            let source = context.child(&facts.types, source, 0)?;
            let target = context.child(&facts.types, target, 0)?;
            context.equate_scoped_terms(&facts.types, source, target)?;
            Ok(Outcome::Ready)
        }
        (Some(Constructor::Dynamic { .. }), Some(Constructor::Dynamic { .. })) => Ok(
            Outcome::Unsupported("source Unsize dyn upcast goals are not implemented"),
        ),
        (_, Some(Constructor::Dynamic { .. })) => {
            let requirements = context.object_requirements(&mut facts.types, source, target)?;
            context.account_work(requirements.len())?;
            if requirements
                .iter()
                .any(|requirement| requirement.bound_regions.is_some())
            {
                return Ok(Outcome::Unsupported(
                    "source Unsize quantified object goals are not implemented",
                ));
            }
            // Trait/projection equations can refine the later builtin inputs.
            // Every requirement is retained; this changes no effect ordering.
            let mut builtins = Vec::new();
            for requirement in requirements {
                if requirement.predicate.builtin_bound().is_some() {
                    builtins.push(requirement.predicate);
                } else {
                    goals.push(requirement.predicate);
                }
            }
            goals.append(&mut builtins);
            Ok(Outcome::Ready)
        }
        (
            Some(Constructor::Adt(..) | Constructor::AdtArgs(..)),
            Some(Constructor::Adt(..) | Constructor::AdtArgs(..)),
        ) => Ok(Outcome::Unsupported(
            "source Unsize nominal-tail goals are not implemented",
        )),
        (None | Some(Constructor::Parameter(_)), _)
        | (_, None | Some(Constructor::Parameter(_))) => Ok(Outcome::Deferred),
        _ => Ok(Outcome::Absent),
    }
}
