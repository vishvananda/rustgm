//! Shared semantic/constant preparation for explicit local runtime roots.
use super::{Prepared, execution};
use crate::{
    resolve::{expansion::Report, model::DefId, package::Error},
    sema::model::Declarations,
};

pub(crate) fn prepare(report: &Report, roots: &[DefId]) -> Result<(Declarations, Prepared), Error> {
    let context = execution::Context::default();
    let mut facts =
        crate::sema::declarations::analyze_local_roots(report, roots, context.executor())?;
    let mut prepared = Prepared::default();
    super::complete_constants(report, &context, &mut facts, &mut prepared)?;
    prepared.constants = Some(context);
    Ok((facts, prepared))
}
