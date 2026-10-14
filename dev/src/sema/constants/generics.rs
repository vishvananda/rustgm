//! Generic const arguments reuse scalar typing, eligibility and evaluation queries.
use super::{Declarations, Error, NodeKey, Queries, Query, Report, Scalar, TypeId, Work};
use crate::resolve::package::ModuleId;
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
