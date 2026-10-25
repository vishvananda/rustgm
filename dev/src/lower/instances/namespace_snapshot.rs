//! The private namespace producer still consumes complete original/body proof.
use super::{diagnostic, inspect_staged};
use crate::{
    resolve::{expansion::Report, package::Error},
    sema::model::Declarations,
    support::crates::{CrateIdentity, generics::source::UnpublishedNamespace},
};
use std::sync::Arc;

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "private namespace transaction awaits qualified module-contract publication; public profile guards remain closed"
    )
)]
pub(crate) fn inspect_source_namespace(
    report: &Report,
    facts: &mut Declarations,
    identity: &CrateIdentity,
    prepared: crate::pipeline::Prepared,
) -> Result<Arc<UnpublishedNamespace>, Error> {
    inspect_staged(
        report,
        facts,
        prepared,
        |facts, checked| {
            let state = crate::lower::transported_source::Source::begin_namespace(
                report, facts, checked, identity,
            )
            .map_err(|e| diagnostic(report, None, e.into()))?;
            let exports = crate::lower::roots::exports(facts, &[])
                .map_err(|e| diagnostic(report, None, e.into()))?;
            Ok((None, exports, state))
        },
        |state, facts, proved, input| {
            state
                .0
                .collect(facts, proved, input)
                .map_err(|e| diagnostic(report, None, e.into()))
        },
        |facts, assembled, roots, state| {
            crate::lower::build_assembled(facts, assembled, roots)
                .map_err(|e| diagnostic(report, e.node, e.message.into()))?;
            state
                .0
                .complete_namespace(state.1, identity)
                .map_err(|e| diagnostic(report, None, e.into()))
        },
    )?
}
