//! Generic const arguments reuse scalar typing, eligibility and evaluation queries.
use super::{Declarations, Error, NodeKey, Queries, Query, Report, Scalar, TypeId, Work};
use crate::resolve::package::ModuleId;

#[derive(Clone, Copy)]
pub(in crate::sema) struct Input {
    pub key: NodeKey,
    pub ty: TypeId,
    pub module: ModuleId,
}

pub(in crate::sema) enum Progress {
    Complete(u128),
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
    ready: &std::collections::HashSet<crate::resolve::model::DefId>,
) -> Result<Progress, Error> {
    let Input { key, ty, module } = input;
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
            queries.typing.start_anonymous_constant(key, ty, module)?
        };
        if let Some(dependency) = dependency {
            let (saved, continuation) = queries.suspend();
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
            let (saved, continuation) = queries.suspend();
            *scratch = saved;
            return Ok(Progress::Prepare(
                super::suspension::Dependency::Members(members),
                continuation,
            ));
        }
    }
    if let Some(definition) = queries.drain_until(Some(ready))? {
        let (saved, continuation) = queries.suspend();
        *scratch = saved;
        return Ok(Progress::Prepare(definition, continuation));
    }
    let scalar = queries
        .value(Query::Node(key))?
        .scalar()
        .map_err(|e| Declarations::error(report, key, e))?;
    *scratch = queries.into_scratch();
    scalar_bits(report, key, scalar).map(Progress::Complete)
}

pub(in crate::sema) fn argument(
    report: &Report,
    facts: &mut Declarations,
    scratch: &mut super::Scratch,
    discriminants: &mut super::Discriminants,
    key: NodeKey,
    ty: TypeId,
    module: ModuleId,
) -> Result<u128, Error> {
    let mut queries =
        Queries::with_scratch(report, facts, Some(discriminants), std::mem::take(scratch));
    queries.value_scope = Some(super::instances::ValueScope::Anonymous { body: key, ty });
    queries.typing.anonymous_constant(key, ty, module)?;
    queries.require_complete_patterns(Query::Node(key))?;
    queries.work.push(Work::Enter(Query::Node(key)));
    queries.prepare(key)?;
    queries.drain()?;
    let scalar = queries
        .value(Query::Node(key))?
        .scalar()
        .map_err(|e| Declarations::error(report, key, e))?;
    *scratch = queries.into_scratch();
    scalar_bits(report, key, scalar)
}

fn scalar_bits(report: &Report, key: NodeKey, scalar: Scalar) -> Result<u128, Error> {
    match scalar {
        Scalar::Integer(bits) => Ok(bits),
        Scalar::Boolean(value) => Ok(u128::from(value)),
        Scalar::Character(value) => Ok(u128::from(u32::from(value))),
        Scalar::Float(_) => Err(Declarations::error(
            report,
            key,
            "floating const generic argument",
        )),
    }
}
