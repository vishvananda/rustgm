//! Selected reconstruction pays one real row before allocating its raw CFG.
use super::{ImportedOriginals, Original, decoded_bodies, rebuilt_inputs, rebuilt_operations};
use crate::{
    resolve::{expansion::Report, model::DefId, package::Error},
    sema::{dispatch::Preparation, model::Declarations},
};
use std::sync::Arc;

pub(crate) fn demand(
    report: &Report,
    facts: &mut Declarations,
    definition: DefId,
    service: &mut dyn Preparation,
) -> Result<Original, Error> {
    let mut work = ImportedOriginals::default();
    let mut allocation = 0;
    let mut reads = false;
    let preflight = service.with_original(facts, definition, &mut |_, view| {
        work.charge(view.raw().values.len().saturating_add(1))
            .map_err(Error::unlocated)?;
        reads = view.raw().values.iter().any(|value| {
            matches!(
                value.kind,
                crate::support::crates::generics::source::ValueKind::Read(_)
            )
        });
        let (bytes, visits) =
            decoded_bodies::allocation(view.rows(), &mut work, 1, std::iter::once(view.raw()))
                .map_err(Error::unlocated)?;
        if visits.saturating_add(work.steps) > crate::support::crates::MAX_COMPONENTS {
            return Err(Error::unlocated(
                "selected original preflight exceeds its actual work budget",
            ));
        }
        allocation = bytes;
        Ok(())
    });
    let charged = service.prepay(facts, work.steps, allocation);
    preflight.and(charged)?;
    if reads {
        service.copy_declarations(facts)?;
    }
    let before = work.steps;
    let mut decoded = None;
    let result = service.with_original(facts, definition, &mut |facts, view| {
        let source = view.source().map_err(Error::unlocated)?;
        let body =
            rebuilt_operations::build(view, &mut work).map_err(|e| report.body_error(source, e))?;
        let contract = rebuilt_inputs::build(facts, view, &body, &mut work)
            .map_err(|e| report.body_error(source, e))?;
        let environment = Arc::clone(
            view.environment()
                .map_err(|e| report.body_error(source, e))?,
        );
        decoded = Some((body, contract, environment));
        Ok(())
    });
    let charged = service.prepay(facts, work.steps.saturating_sub(before), 0);
    result.and(charged)?;
    let (body, contract, environment) = decoded.ok_or_else(|| {
        Error::unlocated("selected original producer omitted its actual raw result")
    })?;
    let mut scratch = crate::body::promote::Scratch::default();
    decoded_bodies::finish(report, facts, &mut scratch, body, contract, environment)
}

pub(crate) fn validate(
    facts: &Declarations,
    view: &crate::sema::crates::SourceOriginal<'_>,
    original: &Original,
    steps: &mut usize,
) -> Result<(), &'static str> {
    rebuilt_operations::validate(view, &original.template.body, steps)?;
    rebuilt_inputs::validate(
        facts,
        view,
        &original.template.body,
        &original.contract,
        steps,
    )
}
