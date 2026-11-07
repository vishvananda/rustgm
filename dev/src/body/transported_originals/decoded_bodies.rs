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
    mut allowance: Allowance,
    rows: s::ValidatedView<'_>,
) -> Result<(), Error> {
    let mut body_bound = preflight(rows, originals, &allowance)
        .map_err(|e| context.report.body_optional_error(None, e))?;
    let before_index = originals.retained_bytes();
    let selected = super::versioned_calls::Index::new(rows, originals, &allowance)
        .map_err(|e| context.report.body_optional_error(None, e))?;
    body_bound = body_bound.saturating_add(originals.retained_bytes().saturating_sub(before_index));
    for raw in &rows.originals {
        originals
            .charge(1)
            .map_err(|e| context.report.body_optional_error(None, e))?;
        let definition = context
            .imported
            .generic_definition(context.owner, raw.declaration)
            .map_err(|e| context.report.body_optional_error(None, e))?;
        crate::sema::crates::prepare_source_original(
            context.report,
            facts,
            definition,
            originals.steps,
            body_bound,
        )?;
        if rows.descriptor().profile() == s::Profile::Source2024ModuleConversions {
            let current = facts
                .crates
                .as_deref()
                .and_then(crate::sema::crates::Linkage::original_work)
                .ok_or_else(|| {
                    context.report.body_optional_error(
                        None,
                        "ordinary original lost its current header accounting",
                    )
                })?;
            allowance.header_work = current.0;
            allowance.header_bytes = current.1;
        }
        let prepaid = body_bound
            .checked_sub(originals.retained_bytes())
            .ok_or_else(|| {
                context.report.body_optional_error(
                    None,
                    "actual original retention exceeded its prepaid bound",
                )
            })?;
        let remaining = Allowance {
            header_bytes: allowance.header_bytes.saturating_add(prepaid),
            templates: allowance.templates.saturating_sub(prepaid),
            ..allowance
        };
        let before_proof = originals.payload;
        let original = build(
            context, facts, scratch, originals, definition, &selected, &remaining,
        )?;
        // Raw body retention is already in the aggregate bound. Only newly
        // charged proof/canonical scratch extends the next preparation's bound.
        body_bound = body_bound.saturating_add(originals.payload.saturating_sub(before_proof));
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
    selected.release(originals);
    Ok(())
}
fn build(
    context: &Import<'_>,
    facts: &mut Declarations,
    scratch: &mut promote::Scratch,
    work: &mut ImportedOriginals,
    definition: crate::resolve::model::DefId,
    selected: &super::versioned_calls::Index<'_>,
    allowance: &Allowance,
) -> Result<Original, Error> {
    let report = context.report;
    let (body, contract, environment, pending, texts) = {
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
        let pending = selected
            .prepare(&view, facts, &body, work)
            .map_err(|e| report.body_error(source, e))?;
        let texts =
            super::encoded_texts::copy(&view, work).map_err(|e| report.body_error(source, e))?;
        (
            body,
            contract,
            Arc::clone(
                view.environment()
                    .map_err(|e| report.body_error(source, e))?,
            ),
            pending,
            texts,
        )
    };
    super::versioned_calls::prove(
        facts,
        Arc::clone(&environment),
        &pending,
        &body,
        &contract,
        work,
        allowance,
    )
    .map_err(|e| report.body_error(body.source, e))?;
    finish(report, facts, scratch, (body, texts), contract, environment)
}
pub(super) fn finish(
    report: &crate::resolve::expansion::Report,
    facts: &mut Declarations,
    scratch: &mut promote::Scratch,
    (mut body, texts): (crate::body::ir::Body, super::encoded_texts::Pending),
    contract: crate::body::regions::Contract,
    environment: Arc<crate::types::parameters::environment::Environment>,
) -> Result<Original, Error> {
    let source = body.source;
    super::encoded_texts::intern(facts, &mut body, texts)
        .map_err(|error| report.body_error(source, error))?;
    super::untupled_calls::check(facts, &body, &contract, &environment)
        .map_err(|error| report.body_error(source, error))?;
    super::variant_facts::check(facts, &body).map_err(|error| report.body_error(source, error))?;
    // A called imported constructor needs its consumer-synthesized template,
    // exactly as a local generic constructor reference requests one. The scan
    // is canonical body work: one step per rebuilt value.
    facts.body_work.canonical_steps = facts
        .body_work
        .canonical_steps
        .saturating_add(body.values.len());
    for value in &body.values {
        if let crate::body::ir::Rvalue::Function(definition) = value.kind
            && facts.members.constructor(definition).is_some()
        {
            facts
                .generics
                .referenced_constructors
                .insert(crate::types::ItemId(definition.0));
        }
    }
    // Imported templates have completed declarations but their exact generic
    // nominal applications still need the ordinary substituted-field demand.
    // This structural preparation supplies no formation or body permission.
    for ty in std::iter::once(body.result)
        .chain(body.locals.iter().map(|local| local.ty))
        .chain(body.values.iter().map(|value| value.ty))
        .chain(body.projections.iter().map(|projection| projection.ty))
    {
        if matches!(facts.types.kind(ty), Some(crate::types::Kind::Adt(..))) {
            facts
                .aggregates
                .prepare(&mut facts.types, ty)
                .map_err(|error| report.body_error(source, error))?;
        }
    }
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
) -> Result<usize, &'static str> {
    let (bytes, visits) = allocation(rows, work, rows.originals.len(), rows.originals.iter())?;
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
    Ok(bytes.saturating_add(work.retained_bytes()))
}

pub(super) fn allocation<'a>(
    rows: s::ValidatedView<'a>,
    work: &mut ImportedOriginals,
    additional: usize,
    originals: impl Iterator<Item = &'a s::Original>,
) -> Result<(usize, usize), &'static str> {
    let mut bytes = 4_usize
        .saturating_mul(work.rows.len().saturating_add(additional))
        .saturating_mul(size_of::<Original>() + size_of::<(crate::resolve::model::DefId, usize)>());
    let mut visits = 0_usize;
    for raw in originals {
        work.charge(1)?;
        let input = rows
            .body_inputs
            .get(rows.original_index(raw.body)?)
            .filter(|input| input.body == raw.body)
            .ok_or("source original preflight omitted its actual raw input owner")?;
        let Some(s::HeaderRef::Function(index)) = rows.header(raw.declaration)? else {
            return Err("source original preflight omitted its actual function header");
        };
        let header = &rows.functions[index as usize];
        if raw.context == s::CallContext::ConstDefinition && !input.parameters.is_empty() {
            bytes = bytes.saturating_add(crate::body::defining_cleanups::required_allocation());
        }
        let clauses = (header.inputs.end - header.inputs.start) as usize;
        bytes = bytes.saturating_add(super::versioned_calls::allocation(
            raw.values.len(),
            rows.descriptor().profile().conversion_roles(),
        ));
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
            .saturating_add(rebuilt_operations::allocation(raw, work)?.saturating_mul(4))
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
    Ok((bytes, visits))
}
