//! Callable equations preserve unresolved inputs and instantiate one signature binder.
mod output;
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    support::call_traits::CallTrait,
    types::{
        RegionId,
        closure::CallKind,
        infer::{self, Constructor, Context, TermId, arguments::Argument, predicates::Predicate},
    },
};
pub(in crate::sema) use output::symbolic;
use std::ops::Range;

pub(in crate::sema) struct Matched {
    pub output: TermId,
    pub regions: Range<u32>,
}

/// Run only in an isolated query. These equations do not grant trait permission
/// until the selector completes the original goal and retains lifetime evidence.
pub(in crate::sema) fn match_signature(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Predicate,
    first: u32,
) -> Result<Matched, infer::Error> {
    let (item, arguments) = goal
        .trait_arguments()
        .ok_or(infer::Error::InvalidIdentity)?;
    let kind = facts
        .associated
        .callable(DefId(item.0))
        .ok_or(infer::Error::InvalidIdentity)?;
    let [Argument::Type(arguments)] = arguments else {
        return Err(infer::Error::InvalidIdentity);
    };
    let signature = signature(facts, context, goal.subject(), kind)?;
    let Some(Constructor::Function {
        bound_regions,
        parameters,
        unsafe_: false,
    }) = context.head(&facts.types, signature)?
    else {
        return Err(infer::Error::Mismatch);
    };
    let end = first
        .checked_add(bound_regions)
        .filter(|end| *end < 0x8000_0000)
        .ok_or(infer::Error::Overflow)?;
    context.account_work(parameters as usize + bound_regions as usize + 1)?;
    let mut children = Vec::with_capacity(parameters as usize + 1);
    for index in 0..=parameters {
        children.push(context.child(&facts.types, signature, index as usize)?);
    }
    // One shared traversal preserves cross-parameter/output correlations and
    // prevents repeated rebuilding of a signature's shared subgraphs.
    let tuple = context.construct(Constructor::Tuple, &children)?;
    let regions = (first..end).map(RegionId).collect::<Vec<_>>();
    let tuple = context.substitute_regions(&mut facts.types, tuple, &regions)?;
    children.clear();
    for index in 0..parameters {
        children.push(context.child(&facts.types, tuple, index as usize)?);
    }
    let inputs = context.construct(Constructor::Tuple, &children)?;
    context.equate_scoped_terms(&facts.types, inputs, *arguments)?;
    Ok(Matched {
        output: context.child(&facts.types, tuple, parameters as usize)?,
        regions: first..end,
    })
}

fn signature(
    facts: &Declarations,
    context: &mut Context,
    subject: TermId,
    required: CallTrait,
) -> Result<TermId, infer::Error> {
    match context.head(&facts.types, subject)? {
        Some(Constructor::Function { .. }) => Ok(subject),
        Some(Constructor::FunctionItem(item, _) | Constructor::FunctionItemArgs(item, _)) => {
            if facts.target_feature_functions.contains(&DefId(item.0)) {
                return Err(infer::Error::Mismatch);
            }
            context.child(&facts.types, subject, 0)
        }
        Some(Constructor::Closure(id, _)) => {
            let required = match required {
                CallTrait::Fn => CallKind::Shared,
                CallTrait::FnMut => CallKind::Mutable,
                CallTrait::FnOnce => CallKind::Once,
            };
            if !facts
                .types
                .closure_kind(id)
                .ok_or(infer::Error::InvalidIdentity)?
                .implements(required)
            {
                return Err(infer::Error::Mismatch);
            }
            context.child(&facts.types, subject, 0)
        }
        None | Some(Constructor::Reference { .. }) => Err(infer::Error::Unresolved),
        Some(_) => Err(infer::Error::Mismatch),
    }
}
