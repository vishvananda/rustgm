//! Whole-source reconstruction stays private until every real original proves.
use super::{
    Import, ImportedOriginals, Original, interface_goals, rebuilt_inputs, rebuilt_operations,
};
use crate::{
    body::{promote, templates::Template},
    resolve::package::Error,
    sema::model::Declarations,
    support::crates::{self, generics::source as s},
};
use std::sync::Arc;

#[derive(Clone, Copy)]
pub(super) struct Allowance {
    pub header_work: usize,
    pub header_bytes: usize,
    pub templates: usize,
}
pub(super) fn append(
    context: &Import<'_>,
    facts: &mut Declarations,
    scratch: &mut promote::Scratch,
    originals: &mut ImportedOriginals,
    allowance: Allowance,
    rows: s::ValidatedView<'_>,
) -> Result<(), Error> {
    preflight(rows, originals, &allowance)
        .map_err(|e| context.report.body_optional_error(None, e))?;
    for raw in &rows.originals {
        originals
            .charge(1)
            .map_err(|e| context.report.body_optional_error(None, e))?;
        let definition = context
            .imported
            .generic_definition(context.owner, raw.declaration)
            .map_err(|e| context.report.body_optional_error(None, e))?;
        let original = build(context, facts, scratch, originals, definition)?;
        let source = original.template.body.source;
        let retained = originals
            .retained_bytes()
            .saturating_add(original.template.body.allocation_bytes())
            .saturating_add(original.template.destruction.allocation_bytes())
            .saturating_add(interface_goals::allocation(&original.contract));
        if allowance.header_work.saturating_add(originals.steps) > crates::MAX_COMPONENTS
            || allowance.header_bytes.saturating_add(retained) > crates::MAX_BYTES
            || retained > allowance.templates
        {
            return Err(context.report.body_error(
                source,
                "source original exceeded its combined preflight envelope",
            ));
        }
        originals
            .retain(original)
            .map_err(|e| context.report.body_error(source, e))?;
    }
    Ok(())
}
fn build(
    context: &Import<'_>,
    facts: &mut Declarations,
    scratch: &mut promote::Scratch,
    work: &mut ImportedOriginals,
    definition: crate::resolve::model::DefId,
) -> Result<Original, Error> {
    let report = context.report;
    let (mut body, contract, environment) = {
        let view = crate::sema::crates::source_original(facts, definition, &mut work.steps)
            .map_err(|e| report.body_optional_error(None, e))?
            .ok_or_else(|| {
                report.body_optional_error(
                    None,
                    "source provided original has no authenticated current data view",
                )
            })?;
        let source = view
            .source()
            .map_err(|e| report.body_optional_error(None, e))?;
        let body =
            rebuilt_operations::build(&view, work).map_err(|e| report.body_error(source, e))?;
        let contract = rebuilt_inputs::build(facts, &view, &body, work)
            .map_err(|e| report.body_error(source, e))?;
        (body, contract, Arc::clone(view.environment()))
    };
    let source = body.source;
    let needed = crate::types::destruction::NeedsDrop::new(
        facts.destructors.implementations().iter().map(|i| i.item),
    );
    let destruction = crate::body::drops::requirements(
        facts,
        &body,
        &mut needed.scoped(Arc::clone(&environment)),
    )
    .map_err(|e| report.body_error(source, e))?;
    promote::instance_reusing(report, facts, &mut body, &destruction, scratch)?;
    crate::body::constants::eligibility_charged(report, facts, &body)?;
    Ok(Original {
        template: Template {
            body,
            destruction,
            environment,
        },
        contract,
    })
}
fn preflight(
    rows: s::ValidatedView<'_>,
    work: &mut ImportedOriginals,
    allowance: &Allowance,
) -> Result<(), &'static str> {
    let mut bytes = 4_usize
        .saturating_mul(work.rows.len().saturating_add(rows.originals.len()))
        .saturating_mul(size_of::<Original>() + size_of::<(crate::resolve::model::DefId, usize)>());
    let mut visits = 0_usize;
    for raw in &rows.originals {
        work.charge(1)?;
        let input = rows
            .body_inputs
            .get(raw.body.0 as usize)
            .filter(|input| input.body == raw.body)
            .ok_or("source original preflight omitted its actual raw input owner")?;
        let Some(s::HeaderRef::Function(index)) = rows.header(raw.declaration)? else {
            return Err("source original preflight omitted its actual function header");
        };
        let header = &rows.functions[index as usize];
        let clauses = (header.inputs.end - header.inputs.start) as usize;
        let ports = raw
            .locals
            .len()
            .saturating_add(raw.values.len())
            .saturating_add(1);
        let owned = raw
            .owned
            .as_ref()
            .map_or(0, |o| o.created.len().saturating_add(o.returned.len()));
        let mut count = ports
            .saturating_add(raw.statements.len())
            .saturating_add(raw.blocks.len())
            .saturating_add(raw.scopes.len())
            .saturating_add(raw.exits.len())
            .saturating_add(raw.temporaries.len())
            .saturating_add(input.parameters.len())
            .saturating_add(input.assumptions.len())
            .saturating_add(input.type_assumptions.len())
            .saturating_add(input.obligations.len())
            .saturating_add(clauses)
            .saturating_add(owned);
        for proof in &input.normalizations {
            work.charge(1)?;
            count = count
                .saturating_add(proof.sources.len())
                .saturating_add(proof.equalities.len())
                .saturating_add(proof.outlives.len())
                .saturating_add(proof.types.len())
                .saturating_add(1);
        }
        if let Some(builtins) = &input.builtins {
            count = count
                .saturating_add(builtins.predicates.len())
                .saturating_add(
                    builtins
                        .source_copy
                        .as_ref()
                        .map_or(0, |copy| copy.goals.len().saturating_add(1)),
                );
        }
        visits = visits.saturating_add(count.saturating_mul(32));
        bytes = bytes
            .saturating_add(rebuilt_operations::allocation(raw).saturating_mul(4))
            .saturating_add(count.saturating_mul(
                8 * (size_of::<crate::body::regions::Obligation>()
                    + size_of::<crate::body::regions::Proof>()
                    + size_of::<(
                        crate::body::ir::ProofSite,
                        crate::types::parameters::predicates::Predicate,
                    )>()),
            ))
            .saturating_add(
                size_of::<crate::body::regions::BuiltinRequirements>() + 4 * size_of::<usize>(),
            );
    }
    if allowance
        .header_work
        .saturating_add(work.steps)
        .saturating_add(visits)
        > crates::MAX_COMPONENTS
        || allowance
            .header_bytes
            .saturating_add(work.retained_bytes())
            .saturating_add(bytes)
            > crates::MAX_BYTES
        || work.retained_bytes().saturating_add(bytes) > allowance.templates
    {
        return Err("source original aggregate work/storage preflight exceeded its budget");
    }
    Ok(())
}
