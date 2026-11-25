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
    let wide = rows.descriptor().profile().declaration_policies();
    let bodies = rows.descriptor().profile().body_nodes();
    prepare_obligations(inputs, port, store, (bodies, wide), work)?;
    if wide {
        prepare_regions(rows, body, inputs, port, store, work)?;
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

fn prepare_obligations<'rows, P: ports::OwnedPort<'rows>>(
    inputs: &s::BodyInputs,
    port: &mut ports::Adapter<'_, P>,
    store: &mut Store,
    (bodies, wide): (bool, bool),
    work: &mut ReservationWork,
) -> Result<(), Error> {
    // Formation roots enter the frontier: an erased primitive operator keeps its
    // selected method's formations, whose function item is that request port.
    work.charge(inputs.obligations.len())?;
    for obligation in &inputs.obligations {
        match obligation.kind {
            s::InputKind::WellFormed(ty) => {
                Port::require_context(&port.port, store, work)?;
                port.ty(store, ty, work)?;
            }
            s::InputKind::ScopedWellFormed { ty, .. } if bodies => {
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
            s::InputKind::LocalBorrow(region) if wide => {
                port.region(region, work)?;
            }
            s::InputKind::Equality(pair) | s::InputKind::Outlives(pair) if wide => {
                port.region(pair.longer, work)?;
                port.region(pair.shorter, work)?;
            }
            s::InputKind::TypeOutlives { ty, shorter } if wide => {
                Port::require_context(&port.port, store, work)?;
                port.ty(store, ty, work)?;
                port.region(shorter, work)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn prepare_regions<'rows, P: ports::OwnedPort<'rows>>(
    rows: s::ValidatedView<'rows>,
    body: BodyRef,
    inputs: &s::BodyInputs,
    port: &mut ports::Adapter<'_, P>,
    store: &mut Store,
    work: &mut ReservationWork,
) -> Result<(), Error> {
    work.charge(
        inputs
            .assumptions
            .len()
            .saturating_add(inputs.normalizations.len())
            .saturating_add(1),
    )?;
    for bound in &inputs.assumptions {
        port.region(bound.longer, work)?;
        port.region(bound.shorter, work)?;
    }
    for record in &inputs.normalizations {
        prepare_relation_rows(
            (&record.equalities, &record.outlives, &record.types),
            port,
            store,
            work,
        )?;
        if let Some(scoped) = &record.scoped {
            for block in &scoped.blocks {
                work.charge(1)?;
                prepare_relation_rows(
                    (&block.equalities, &block.outlives, &block.types),
                    port,
                    store,
                    work,
                )?;
            }
        }
    }
    let original = rows
        .originals
        .get(rows.original_index(body).map_err(error)?)
        .filter(|original| original.body == body)
        .ok_or_else(|| error("ordinary requests lost their actual original body"))?;
    work.charge(
        original
            .projections
            .len()
            .saturating_add(original.openings.len()),
    )?;
    for projection in &original.projections {
        if let s::ProjectionKind::Dereference { region, .. } = projection.kind {
            port.region(region, work)?;
        }
    }
    for &region in &original.openings {
        port.region(region, work)?;
    }
    Ok(())
}

fn prepare_relation_rows<'rows, P: ports::OwnedPort<'rows>>(
    (equalities, outlives, types): (&[s::RegionBound], &[s::RegionBound], &[s::TypeOutlives]),
    port: &mut ports::Adapter<'_, P>,
    store: &mut Store,
    work: &mut ReservationWork,
) -> Result<(), Error> {
    work.charge(
        equalities
            .len()
            .saturating_add(outlives.len())
            .saturating_add(types.len()),
    )?;
    for bound in equalities.iter().chain(outlives) {
        port.region(bound.longer, work)?;
        port.region(bound.shorter, work)?;
    }
    for row in types {
        Port::require_context(&port.port, store, work)?;
        port.ty(store, row.ty, work)?;
        port.region(row.shorter, work)?;
    }
    Ok(())
}
