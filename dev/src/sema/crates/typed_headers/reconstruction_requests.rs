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
        .get(body.0 as usize)
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
        .map(|input| input.value)
        .chain(copies.iter().copied())
        .chain(inputs.type_assumptions.iter().map(|input| input.value))
    {
        // Even a ready application is data in this exact current body/domain.
        Port::require_context(&port.port, store, work)?;
        match requirement {
            s::Requirement::Trait { application, .. } => {
                translation::application(port, applications, application, store, work)?;
            }
            s::Requirement::TypeOutlives { subject, shorter } => {
                port.ty(store, subject, work)?;
                port.region(shorter, work)?;
            }
            s::Requirement::Outlives(_) => {
                return Err(error(
                    "regional clauses require their separate header relation remap",
                ));
            }
        }
    }
    Ok(())
}
