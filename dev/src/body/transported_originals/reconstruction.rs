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
    demand_with(report, facts, service, |service, facts, consume| {
        service.with_original(facts, definition, consume)
    })
}

pub(crate) fn demand_closure(
    report: &Report,
    facts: &mut Declarations,
    identity: crate::types::closure::ClosureId,
    service: &mut dyn Preparation,
) -> Result<Original, Error> {
    demand_with(report, facts, service, |service, facts, consume| {
        service.with_closure(facts, identity, consume)
    })
}

fn demand_with(
    report: &Report,
    facts: &mut Declarations,
    service: &mut dyn Preparation,
    mut access: impl FnMut(
        &mut dyn Preparation,
        &mut Declarations,
        &mut dyn FnMut(&Declarations, &crate::sema::crates::SourceOriginal<'_>) -> Result<(), Error>,
    ) -> Result<(), Error>,
) -> Result<Original, Error> {
    let mut work = ImportedOriginals::default();
    let mut allocation = 0;
    let mut reads = false;
    let preflight = access(service, facts, &mut |_, view| {
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
    let result = access(service, facts, &mut |facts, view| {
        let source = view.source().map_err(Error::unlocated)?;
        let body =
            rebuilt_operations::build(view, &mut work).map_err(|e| report.body_error(source, e))?;
        let (contract, quantified_assumptions) =
            rebuilt_inputs::build(facts, view, &body, &mut work)
                .map_err(|e| report.body_error(source, e))?;
        let environment = Arc::clone(
            view.environment()
                .map_err(|e| report.body_error(source, e))?,
        );
        let texts = super::encoded_texts::copy(view, &mut work)
            .map_err(|e| report.body_error(source, e))?;
        let caller_sites = super::callsite_coordinates::copy(view, &mut work)
            .map_err(|e| report.body_error(source, e))?;
        decoded = Some((
            decoded_bodies::PendingBody {
                body,
                texts,
                caller_sites,
                quantified_assumptions,
            },
            contract,
            environment,
        ));
        Ok(())
    });
    let charged = service.prepay(facts, work.steps.saturating_sub(before), 0);
    result.and(charged)?;
    let (body, contract, environment) = decoded.ok_or_else(|| {
        Error::unlocated("selected original producer omitted its actual raw result")
    })?;
    let mut scratch = crate::body::promote::Scratch::default();
    let mut original = decoded_bodies::finish(
        report,
        facts,
        &mut scratch,
        body,
        contract,
        environment,
        Some(service),
    )?;
    let source = original.template.body.source;
    if facts
        .inline_constants
        .as_deref()
        .is_some_and(|inline| inline.has_children_origin(source))
    {
        decoded_bodies::complete_inline_children(report, facts, &mut scratch, source)?;
        crate::body::instantiate::close_inline_operands(facts, &mut original.template.body)
            .map_err(|e| report.body_error(source, e))?;
    }
    Ok(original)
}

pub(crate) fn validate(
    facts: &Declarations,
    view: &crate::sema::crates::SourceOriginal<'_>,
    original: &Original,
    steps: &mut usize,
) -> Result<(), &'static str> {
    rebuilt_operations::validate(facts, view, &original.template.body, steps)?;
    rebuilt_inputs::validate(
        facts,
        view,
        &original.template.body,
        &original.contract,
        steps,
    )
}
