//! Nonzero signatures use only neutral defining construction, never regional solving.
use super::{
    Arc, Body, Declarations, Error, Owner, Prepared, Ready, RecipeView, ReservationWork, error,
    numbering_records, originals, ports, s,
};
use crate::{sema::function_context::assumption_environment as defining, types::Kind};

pub(super) fn signature(
    rows: s::ValidatedView<'_>,
    expected: Ready,
    work: &mut ReservationWork,
) -> Result<bool, Error> {
    work.charge(8)?;
    let header = expected.header;
    if header.regions.captures != 0
        || !header
            .regions
            .early
            .slice(&rows.region_slots)
            .map_err(error)?
            .is_empty()
        || !header
            .regions
            .bounds
            .slice(&rows.region_bounds)
            .map_err(error)?
            .is_empty()
    {
        return Ok(false);
    }
    for (signature, kind, canonical) in [
        (
            header.original,
            s::BinderKind::OriginalFunction,
            expected.original,
        ),
        (
            header.callable,
            s::BinderKind::PublicFunction,
            expected.callable,
        ),
    ] {
        let frame = rows
            .binders
            .get(signature.binder.0 as usize)
            .ok_or_else(|| error("neutral numbered signature lost its actual binder"))?;
        let Kind::Function { bound_regions, .. } = canonical else {
            return Err(error(
                "neutral numbered signature lost its canonical function",
            ));
        };
        if frame.owner != header.declaration || frame.kind != kind || frame.count != bound_regions {
            return Err(error("neutral numbered signature changed its actual frame"));
        }
        if frame.parent.is_some() || frame.count != header.regions.total {
            return Ok(false);
        }
    }
    let late = header
        .regions
        .late
        .slice(&rows.region_slots)
        .map_err(error)?;
    work.charge(late.len())?;
    for reference in late {
        let slot = rows
            .slots
            .get(reference.0 as usize)
            .ok_or_else(|| error("neutral numbered signature lost its actual late slot"))?;
        if slot.owner != header.declaration || slot.kind != s::ParameterKind::Region {
            return Err(error(
                "neutral numbered signature changed its late slot owner",
            ));
        }
        if slot.definition.is_some() {
            return Ok(false);
        }
    }
    Ok(true)
}
impl Owner {
    // A route hint only: cached_numbered still authenticates every seal/key.
    pub(super) fn numbered_hint(
        &self,
        expected: Ready,
        work: &mut ReservationWork,
    ) -> Result<bool, Error> {
        work.charge(1)?;
        Ok(
            matches!(self.bodies.get(expected.pair.function_row as usize).and_then(Option::as_deref),
            Some(Body::Numbered { expected: actual, .. }) if *actual == expected),
        )
    }
    pub(super) fn finish_numbered(
        &self,
        port: &ports::Inputs<'_, '_>,
        facts: &Declarations,
        prepared: &Prepared,
        demand: &originals::BodyDemand,
        kernel: defining::Prepared,
        work: &mut ReservationWork,
    ) -> Result<numbering_records::Completed, Error> {
        work.charge(2)?;
        let scope = self
            .scopes
            .get(demand.expected.scope.0)
            .and_then(std::cell::OnceCell::get)
            .map(Box::as_ref)
            .ok_or_else(|| error("neutral numbered body lost its actual declaration"))?;
        prepared
            .recordings
            .as_deref()
            .ok_or_else(|| error("neutral numbered body lost its actual recorded domain"))?
            .finish_neutral(
                &numbering_records::Request {
                    view: port.port.recipe_view(),
                    store: &facts.types,
                    expected: &demand.expected,
                    declaration: &scope.environment,
                },
                kernel,
                work,
            )
    }
    pub(super) fn cached_numbered(
        &self,
        body: &Body,
        view: &RecipeView<'_>,
        facts: &Declarations,
        prepared: &Prepared,
        demand: &originals::BodyDemand,
        work: &mut ReservationWork,
    ) -> Result<(), Error> {
        let Body::Numbered {
            expected,
            scope,
            product,
        } = body
        else {
            return Err(error(
                "neutral body cache changed its actual recorded variant",
            ));
        };
        let rows = view.rows()?;
        let declaration = self
            .scopes
            .get(*scope)
            .and_then(std::cell::OnceCell::get)
            .map(Box::as_ref)
            .ok_or_else(|| error("neutral numbered body lost its actual declaration"))?;
        declaration.require(rows, facts)?;
        if *expected != demand.expected
            || declaration.raw != expected.scope.1
            || declaration.parameters.len() != expected.formals
            || !Arc::ptr_eq(&product.declaration, &declaration.environment)
        {
            return Err(error(
                "neutral numbered body changed its actual declaration Arc",
            ));
        }
        prepared
            .recordings
            .as_deref()
            .ok_or_else(|| error("neutral numbered body lost its actual recording owner"))?
            .require_neutral(
                &numbering_records::Request {
                    view,
                    store: &facts.types,
                    expected,
                    declaration: &declaration.environment,
                },
                &product.recording,
                &product.environment,
                work,
            )
    }
}

#[cfg(test)]
#[path = "../../../../tests/invariants/neutral_body_inputs.rs"]
pub(super) mod tests;
