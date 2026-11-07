//! Generic const arguments reuse ordinary typing, eligibility and evaluated values.
mod valtrees;
use super::{Declarations, Error, NodeKey, Queries, Query, Report, TypeId, Work};
use crate::resolve::package::ModuleId;
use crate::types::arguments::ConstId;

#[derive(Clone, Copy)]
pub(in crate::sema) struct Input {
    pub key: NodeKey,
    pub ty: TypeId,
    pub module: ModuleId,
    pub privacy: crate::sema::bodies::TypePrivacy,
}

pub(in crate::sema) enum Progress {
    Complete(ConstId),
    Prepare(
        super::suspension::Dependency,
        Box<super::suspension::Continuation>,
    ),
}

pub(in crate::sema) fn advance(
    report: &Report,
    facts: &mut Declarations,
    scratch: &mut super::Scratch,
    discriminants: &mut super::Discriminants,
    input: Input,
    continuation: Option<Box<super::suspension::Continuation>>,
    ready: &crate::support::hash::HashSet<crate::resolve::model::DefId>,
) -> Result<Progress, Error> {
    let Input {
        key,
        ty,
        module,
        privacy,
    } = input;
    scratch.typing.resumable();
    let mut queries =
        Queries::with_scratch(report, facts, Some(discriminants), std::mem::take(scratch));
    // Declaration bodies may prove a goal once its compatible candidate inventory
    // is complete. A pending header yields through the body's owned continuation.
    queries.typing.availability = crate::sema::traits::implementations::Availability::Declarations;
    queries.value_scope = Some(super::instances::ValueScope::Anonymous { body: key, ty });
    let fresh = continuation.is_none();
    if let Some(continuation) = continuation {
        queries.resume(*continuation);
    }
    if fresh || queries.typing.pending_root() {
        let dependency = if queries.typing.pending_root() {
            queries.typing.advance_body()?
        } else {
            queries
                .typing
                .start_anonymous_constant(key, ty, module, privacy)?
        };
        if let Some(dependency) = dependency {
            let (saved, continuation) = queries.suspend()?;
            *scratch = saved;
            return Ok(Progress::Prepare(
                super::suspension::Dependency::Body(dependency),
                continuation,
            ));
        }
        queries.require_complete_patterns(Query::Node(key))?;
        queries.work.push(Work::Enter(Query::Node(key)));
        queries.work.push(Work::PrepareNode(key));
        let instances = queries.typing.constant_instances()?;
        let members = queries.demand_members(instances, None)?;
        if !members.is_empty() {
            let (saved, continuation) = queries.suspend()?;
            *scratch = saved;
            return Ok(Progress::Prepare(
                super::suspension::Dependency::Members(members),
                continuation,
            ));
        }
    }
    if let Some(definition) = queries.drain_until(Some(ready))? {
        let (saved, continuation) = queries.suspend()?;
        *scratch = saved;
        return Ok(Progress::Prepare(definition, continuation));
    }
    let result = complete(&mut queries, key, ty)?;
    *scratch = queries.into_scratch();
    Ok(Progress::Complete(result))
}

pub(in crate::sema) fn argument(
    report: &Report,
    facts: &mut Declarations,
    scratch: &mut super::Scratch,
    discriminants: &mut super::Discriminants,
    input: Input,
) -> Result<ConstId, Error> {
    let Input {
        key,
        ty,
        module,
        privacy,
    } = input;
    let mut queries =
        Queries::with_scratch(report, facts, Some(discriminants), std::mem::take(scratch));
    queries.value_scope = Some(super::instances::ValueScope::Anonymous { body: key, ty });
    queries
        .typing
        .anonymous_constant(key, ty, module, privacy)?;
    queries.require_complete_patterns(Query::Node(key))?;
    queries.work.push(Work::Enter(Query::Node(key)));
    queries.prepare(key)?;
    queries.drain()?;
    let result = complete(&mut queries, key, ty)?;
    *scratch = queries.into_scratch();
    Ok(result)
}

fn complete(queries: &mut Queries<'_>, key: NodeKey, ty: TypeId) -> Result<ConstId, Error> {
    let value = queries.value(Query::Node(key))?;
    // Value graph work has its own finite phase budget; it is not bounded by AST size.
    let value_limit = queries
        .steps
        .saturating_add(crate::support::crates::MAX_COMPONENTS);
    valtrees::canonicalize(
        queries.typing.facts,
        &mut queries.steps,
        value_limit,
        ty,
        super::values::Constant(value),
    )
    .map_err(|e| Declarations::error(queries.report, key, e))
}
