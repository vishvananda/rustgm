//! Observe the real v14 declaration checks at the publication boundary.
use super::{Construction, Declarations, Error, Report, diagnostic, inspect_staged};
use crate::{pipeline::Prepared, sema::crates::authored_export, support::crates::CrateIdentity};
pub use authored_export::{DeclCensus, DeclFailure};
use std::{convert::Infallible, ops::ControlFlow};

/// Census v14 declaration admission after ordinary source and template checks.
/// # Errors
/// Propagates the ordinary proof, preflight and budget errors.
pub fn inspect_declaration_exports(
    report: &Report,
    facts: &mut Declarations,
    identity: &CrateIdentity,
    prepared: Prepared,
) -> Result<DeclCensus, Error> {
    inspect_staged::<Infallible, _>(
        report,
        facts,
        prepared,
        |facts, _| {
            facts.require_deprecated_safe_transport()?;
            authored_export::prepare_inherent_constants(report, facts, identity)?;
            let census = DeclCensus::collect(report, facts, identity)
                .map_err(|message| diagnostic(report, None, message.into()))?;
            Ok(ControlFlow::Break(census))
        },
        |never, _, _, _| match *never {},
        |_, _, _, never| match never {},
        Construction::CompilerOperations,
    )
}
