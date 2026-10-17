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
        crate::resolve::model::DefId,
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
    let mut queries =
        Queries::with_scratch(report, facts, Some(discriminants), std::mem::take(scratch));
    queries.value_scope = Some((key, ty));
    if let Some(continuation) = continuation {
        queries.resume(*continuation);
    } else {
        queries.typing.anonymous_constant(key, ty, module)?;
        queries.require_complete_patterns(Query::Node(key))?;
        queries.work.push(Work::Enter(Query::Node(key)));
        queries.prepare(key)?;
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
    queries.value_scope = Some((key, ty));
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
