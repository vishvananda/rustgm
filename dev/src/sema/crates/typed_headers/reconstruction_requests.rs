//! Body-only applications enter the same canonical cache before raw reconstruction.
use super::{Error, ReservationWork, error, ports, s};
use crate::{
    sema::crates::typed_headers::{
        predicate_remap::translation::{self, Mapping},
        recipe_cursor::Port,
    },
    support::crates::owner_catalog::BodyRef,
    types::Store,
};

pub(super) fn prepare<'rows, P: ports::OwnedPort<'rows>>(
    body: BodyRef,
    port: &mut ports::Adapter<'_, P>,
    applications: &mut impl translation::applications::Cache,
    store: &mut Store,
    work: &mut ReservationWork,
) -> Result<(), Error> {
    let rows = Port::rows(&port.port);
    let inputs = rows
        .body_inputs
        .get(rows.original_index(body).map_err(error)?)
        .filter(|input| input.body == body)
        .ok_or_else(|| error("selected requests lost their actual raw body inputs"))?;
    let builtins = inputs.builtins.as_ref();
    let predicates = builtins.map_or(&[][..], |input| input.predicates.as_slice());
    let copies = builtins
        .and_then(|input| input.source_copy.as_ref())
        .map_or(&[][..], |input| input.goals.as_slice());
    work.charge(
        predicates
            .len()
            .saturating_add(copies.len())
            .saturating_add(inputs.type_assumptions.len())
            .saturating_add(1),
    )?;
    for requirement in predicates
        .iter()
        .map(|input| &input.value)
        .chain(copies.iter())
        .chain(inputs.type_assumptions.iter().map(|input| &input.value))
    {
        // Even a ready application is data in this exact current body/domain.
        Port::require_context(&port.port, store, work)?;
        match requirement {
            s::Requirement::Trait { application, .. } => {
                translation::application(port, applications, *application, store, work)?;
            }
            s::Requirement::TypeOutlives { subject, shorter } => {
                port.ty(store, *subject, work)?;
                port.region(*shorter, work)?;
            }
            s::Requirement::ProjectionEqual { projection, value } => {
                port.ty(store, *projection, work)?;
                port.ty(store, *value, work)?;
            }
            s::Requirement::Outlives(_) => {
                return Err(error(
                    "regional clauses require their separate header relation remap",
                ));
            }
            s::Requirement::UnstableFeature(_) => {
                return Err(error(
                    "feature clauses cannot be reconstructed as type/Copy body requests",
                ));
            }
        }
    }
    // Formation roots enter the frontier: an erased primitive operator keeps its
    // selected method's formations, whose function item is that request port.
    work.charge(inputs.obligations.len())?;
    let bodies = rows.descriptor().profile().body_nodes();
    for obligation in &inputs.obligations {
        match obligation.kind {
            s::InputKind::WellFormed(ty) => {
                Port::require_context(&port.port, store, work)?;
                port.ty(store, ty, work)?;
            }
            // Version thirteen: a coercion's types may occur only in its
            // input rows, as when a static literal flows into a merge.
            s::InputKind::Coercion {
                actual, expected, ..
            } if bodies => {
                Port::require_context(&port.port, store, work)?;
                port.ty(store, actual, work)?;
                port.ty(store, expected, work)?;
            }
            _ => {}
        }
    }
    if rows.descriptor().profile().conversion_roles() {
        let original = rows
            .originals
            .get(rows.original_index(body).map_err(error)?)
            .filter(|original| original.body == body)
            .ok_or_else(|| error("ordinary requests lost their actual original body"))?;
        work.charge(original.calls.len().saturating_add(1))?;
        for call in &original.calls {
            if let Some(application) = call.method {
                Port::require_context(&port.port, store, work)?;
                translation::application(port, applications, application, store, work)?;
            }
        }
    }
    Ok(())
}
