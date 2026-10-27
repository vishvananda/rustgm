//! Encoded provided consumers keep their selected producers inside one execution.
use crate::{
    resolve::{
        crates::{ConsumerRequest, ReservationWork},
        expansion::Report,
        load::Sources,
        package::Error,
    },
    sema::{crates::ProofSession, model::Declarations},
};

/// Resolve and prove an executable against a decoded provided-contract artifact.
/// The callback borrows the same consumer after original and fresh instance proof.
/// # Errors
/// Preserves loading, declaration, body, region, ownership and loan diagnostics.
pub fn with_provided_client<R>(
    request: ConsumerRequest<'_, impl Sources>,
    inspect: impl FnOnce(&Report, &Declarations, &crate::lowir::program::Program) -> R,
) -> Result<R, Error> {
    let mut work = ReservationWork::default();
    let draft = ProofSession::from_encoded(request, &mut work)?;
    let (_, result) = draft.execute(&mut work, |report, facts, entry, preparation| {
        let program = crate::lower::instances::build_selected_prepared(
            report,
            facts,
            entry,
            crate::pipeline::Prepared::default(),
            preparation,
        )?;
        Ok(inspect(report, facts, &program))
    })?;
    Ok(result)
}
