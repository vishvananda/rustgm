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
    let start_row = originals.rows.len();
    for raw in &rows.originals {
        originals
            .charge(1)
            .map_err(|e| context.report.body_optional_error(None, e))?;
        let target = prepare_target(context, facts, rows, raw, &mut originals.steps, body_bound)?;
        let profile = rows.descriptor().profile();
        if profile == s::Profile::Source2024ModuleConversions || profile.declaration_policies() {
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
            context, facts, scratch, originals, target, &selected, &remaining,
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
    if let Some(inline) = facts.inline_constants.as_deref() {
        for row in &mut originals.rows[start_row..] {
            if inline.has_children_origin(row.template.body.source) {
                crate::body::instantiate::close_inline_operands(facts, &mut row.template.body)
                    .map_err(|e| context.report.body_error(row.template.body.source, e))?;
            }
        }
    }
    selected.release(originals);
    Ok(())
}
pub(super) struct PendingBody {
    pub(super) body: crate::body::ir::Body,
    pub(super) texts: super::encoded_texts::Pending,
    pub(super) caller_sites: super::callsite_coordinates::Pending,
    pub(super) quantified_assumptions: Vec<rebuilt_inputs::QuantifiedAssumption>,
}

#[derive(Clone, Copy)]
pub(crate) enum Target {
    Declaration(crate::resolve::model::DefId),
    Closure(crate::types::closure::ClosureId),
    Anonymous(crate::resolve::body_origin::ImportedBodyId),
}

pub(crate) fn demand_target(
    report: &crate::resolve::expansion::Report,
    facts: &mut Declarations,
    target: Target,
    scratch: &mut promote::Scratch,
) -> Result<Original, Error> {
    let mut work = ImportedOriginals::default();
    match target {
        Target::Declaration(definition) => {
            crate::sema::crates::prepare_source_original(report, facts, definition, 0, 0)?;
        }
        Target::Closure(identity) => {
            crate::sema::crates::prepare_source_closure_original(report, facts, identity, 0, 0)?;
        }
        Target::Anonymous(body_id) => {
            crate::sema::crates::prepare_source_anonymous_original(report, facts, body_id, 0, 0)?;
        }
    }
    let (rebuilt, contract, environment) = {
        let view = match target {
            Target::Declaration(definition) => {
                crate::sema::crates::source_original(facts, definition, &mut work.steps)
            }
            Target::Closure(identity) => {
                crate::sema::crates::source_closure_original(facts, identity, &mut work.steps)
            }
            Target::Anonymous(body_id) => {
                crate::sema::crates::source_anonymous_original(facts, body_id, &mut work.steps)
            }
        }
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
        let body = rebuilt_operations::build(&view, &mut work)
            .map_err(|e| report.body_error(source, e))?;
        let (contract, quantified_assumptions) =
            rebuilt_inputs::build(facts, &view, &body, &mut work)
                .map_err(|e| report.body_error(source, e))?;
        let texts = super::encoded_texts::copy(&view, &mut work)
            .map_err(|e| report.body_error(source, e))?;
        let caller_sites = super::callsite_coordinates::copy(&view, &mut work)
            .map_err(|e| report.body_error(source, e))?;
        (
            PendingBody {
                body,
                texts,
                caller_sites,
                quantified_assumptions,
            },
            contract,
            Arc::clone(
                view.environment()
                    .map_err(|e| report.body_error(source, e))?,
            ),
        )
    };
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work.steps);
    let mut original = finish(report, facts, scratch, rebuilt, contract, environment, None)?;
    let source = original.template.body.source;
    if facts
        .inline_constants
        .as_deref()
        .is_some_and(|inline| inline.has_children_origin(source))
    {
        complete_inline_children(report, facts, scratch, source)?;
        crate::body::instantiate::close_inline_operands(facts, &mut original.template.body)
            .map_err(|e| report.body_error(source, e))?;
    }
    Ok(original)
}

pub(super) fn complete_inline_children(
    report: &crate::resolve::expansion::Report,
    facts: &mut Declarations,
    scratch: &mut promote::Scratch,
    source: crate::body::ir::BodyOrigin,
) -> Result<(), Error> {
    let pending = facts
        .inline_constants
        .as_deref()
        .map(|inline| inline.uncompleted_imported_children(source))
        .unwrap_or_default();
    for child in pending {
        let _ = demand_target(report, facts, Target::Anonymous(child), scratch)?;
    }
    Ok(())
}

fn prepare_target(
    context: &Import<'_>,
    facts: &mut Declarations,
    rows: s::ValidatedView<'_>,
    raw: &s::Original,
    steps: &mut usize,
    body_bound: usize,
) -> Result<Target, Error> {
    let target = if rows
        .closures()
        .and_then(|closures| closures.header_for_body(raw.body))
        .is_some()
    {
        let identity =
            crate::sema::crates::source_closure_identity(facts, context.owner, raw.body, steps)
                .map_err(|e| context.report.body_optional_error(None, e))?
                .ok_or_else(|| {
                    context.report.body_optional_error(
                        None,
                        "source closure original lost its canonical identity",
                    )
                })?;
        crate::sema::crates::prepare_source_closure_original(
            context.report,
            facts,
            identity,
            *steps,
            body_bound,
        )?;
        Target::Closure(identity)
    } else if rows
        .closures()
        .and_then(|closures| closures.anonymous_for_body(raw.body))
        .is_some()
    {
        let body_id = context
            .catalog
            .body_at(context.owner, raw.body)
            .map_err(|e| context.report.body_optional_error(None, e))?;
        crate::sema::crates::prepare_source_anonymous_original(
            context.report,
            facts,
            body_id,
            *steps,
            body_bound,
        )?;
        Target::Anonymous(body_id)
    } else {
        let definition = context
            .imported
            .generic_definition(context.owner, raw.declaration)
            .map_err(|e| context.report.body_optional_error(None, e))?;
        crate::sema::crates::prepare_source_original(
            context.report,
            facts,
            definition,
            *steps,
            body_bound,
        )?;
        Target::Declaration(definition)
    };
    Ok(target)
}

fn build(
    context: &Import<'_>,
    facts: &mut Declarations,
    scratch: &mut promote::Scratch,
    work: &mut ImportedOriginals,
    target: Target,
    selected: &super::versioned_calls::Index<'_>,
    allowance: &Allowance,
) -> Result<Original, Error> {
    let report = context.report;
    let (rebuilt, mut contract, environment, pending, dependency_inputs) = {
        let view = match target {
            Target::Declaration(definition) => {
                crate::sema::crates::source_original(facts, definition, &mut work.steps)
            }
            Target::Closure(identity) => {
                crate::sema::crates::source_closure_original(facts, identity, &mut work.steps)
            }
            Target::Anonymous(body_id) => {
                crate::sema::crates::source_anonymous_original(facts, body_id, &mut work.steps)
            }
        }
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
        let (contract, quantified_assumptions) = rebuilt_inputs::build(facts, &view, &body, work)
            .map_err(|e| report.body_error(source, e))?;
        let pending = selected
            .prepare(&view, facts, &body, work)
            .map_err(|e| report.body_error(source, e))?;
        let dependency_inputs = super::external_call_premises::prepare(&view, &body, work)
            .map_err(|e| report.body_error(source, e))?;
        let texts =
            super::encoded_texts::copy(&view, work).map_err(|e| report.body_error(source, e))?;
        let caller_sites = super::callsite_coordinates::copy(&view, work)
            .map_err(|e| report.body_error(source, e))?;
        (
            PendingBody {
                body,
                texts,
                caller_sites,
                quantified_assumptions,
            },
            contract,
            Arc::clone(
                view.environment()
                    .map_err(|e| report.body_error(source, e))?,
            ),
            pending,
            dependency_inputs,
        )
    };
    super::versioned_calls::prove(
        facts,
        Arc::clone(&environment),
        &pending,
        &rebuilt.body,
        &mut contract,
        work,
        allowance,
    )
    .map_err(|e| report.body_error(rebuilt.body.source, e))?;
    super::external_call_premises::check(facts, &dependency_inputs, &contract, work)
        .map_err(|e| report.body_error(rebuilt.body.source, e))?;
    finish(report, facts, scratch, rebuilt, contract, environment, None)
}
pub(super) fn finish(
    report: &crate::resolve::expansion::Report,
    facts: &mut Declarations,
    scratch: &mut promote::Scratch,
    rebuilt: PendingBody,
    mut contract: crate::body::regions::Contract,
    environment: Arc<crate::types::parameters::environment::Environment>,
    preparation: Option<&mut dyn crate::sema::dispatch::Preparation>,
) -> Result<Original, Error> {
    let PendingBody {
        mut body,
        texts,
        caller_sites,
        quantified_assumptions,
    } = rebuilt;
    let source = body.source;
    for assumption in quantified_assumptions {
        let id = facts
            .types
            .intern_quantified(crate::types::parameters::clauses::Quantified {
                bound_regions: assumption.bound_regions,
                predicate: assumption.predicate,
            })
            .map_err(|error| report.body_error(source, error))?;
        let slot = contract
            .type_assumptions
            .get_mut(assumption.index)
            .ok_or_else(|| {
                report.body_error(source, "quantified original assumption index out of bounds")
            })?;
        *slot = crate::types::parameters::clauses::Clause::ForAll(id);
    }
    super::encoded_texts::intern(facts, &mut body, texts)
        .map_err(|error| report.body_error(source, error))?;
    super::callsite_coordinates::intern(report, facts, &mut body, caller_sites)
        .map_err(|error| report.body_error(source, error))?;
    if crate::support::sealed::self_check() {
        super::untupled_calls::check(facts, &body, &contract, &environment)
            .map_err(|error| report.body_error(source, error))?;
        super::variant_facts::check(facts, &body)
            .map_err(|error| report.body_error(source, error))?;
    }
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
    super::remapped_field_types::prepare_storage(facts, &body, preparation)
        .map_err(|error| report.body_error(source, error))?;
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
    if matches!(body.owner, crate::body::ir::Owner::AnonymousConstant) {
        facts
            .inline_constants
            .as_mut()
            .ok_or_else(|| report.body_error(source, "imported anonymous registry disappeared"))?
            .complete_imported(source, body.result)
            .map_err(|e| report.body_error(source, e))?;
    }
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
        .saturating_mul(
            size_of::<Original>()
                + size_of::<(crate::resolve::model::DefId, usize)>()
                + size_of::<(crate::types::closure::ClosureId, usize)>()
                + size_of::<(crate::resolve::body_origin::ImportedBodyId, usize)>(),
        );
    let mut visits = 0_usize;
    for raw in originals {
        work.charge(1)?;
        let input = rows
            .body_inputs
            .get(rows.original_index(raw.body)?)
            .filter(|input| input.body == raw.body)
            .ok_or("source original preflight omitted its actual raw input owner")?;
        let (clauses, closure_items, closure_bytes) = header_allocation(rows, raw)?;
        bytes = bytes.saturating_add(closure_bytes);
        if raw.context != s::CallContext::Runtime && !input.parameters.is_empty() {
            bytes = bytes.saturating_add(crate::body::defining_cleanups::required_allocation());
        }
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
            .saturating_add(closure_items)
            .saturating_add(owned);
        count = count.saturating_add(normalization_count(&input.normalizations, work)?);
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
        let (operation_bytes, associated) = rebuilt_operations::allocation(raw, work)?;
        let (site_bytes, site_visits) = super::callsite_coordinates::allocation(rows, raw.body);
        bytes = bytes.saturating_add(site_bytes);
        count = count.saturating_add(associated).saturating_add(site_visits);
        visits = visits.saturating_add(count.saturating_mul(32));
        bytes = bytes
            .saturating_add(operation_bytes.saturating_mul(4))
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

fn header_allocation(
    rows: s::ValidatedView<'_>,
    raw: &s::Original,
) -> Result<(usize, usize, usize), &'static str> {
    if let Some((_, closure)) = rows
        .closures()
        .and_then(|closures| closures.header_for_body(raw.body))
    {
        let (bytes, items) = nested_output_allocation(closure.output);
        return Ok((0, items, bytes));
    }
    if let Some((_, anonymous)) = rows
        .closures()
        .and_then(|closures| closures.anonymous_for_body(raw.body))
    {
        let (bytes, items) = nested_output_allocation(anonymous.output);
        return Ok((0, items, bytes));
    }
    let clauses = match rows.header(raw.declaration)? {
        Some(s::HeaderRef::Function(index)) => {
            let inputs = rows.functions[index as usize].inputs;
            (inputs.end - inputs.start) as usize
        }
        Some(s::HeaderRef::Constant(index))
            if rows.descriptor().profile().declaration_policies()
                && rows.constants()[index as usize].body == s::ConstantBody::Original(raw.body)
                && raw.context == s::CallContext::ClosedConst =>
        {
            0
        }
        _ => return Err("source original preflight omitted its actual defining header"),
    };
    Ok((clauses, 0, 0))
}

fn nested_output_allocation(output: Option<s::ClosureOutput>) -> (usize, usize) {
    let Some(output) = output else {
        return (0, 0);
    };
    let mut items = (output.bridges.end - output.bridges.start) as usize;
    if let Some(formation) = output.formation {
        items = items
            .saturating_add((formation.inputs.end - formation.inputs.start) as usize)
            .saturating_add(
                (formation.callable_slots.end - formation.callable_slots.start) as usize,
            );
    }
    let bytes = size_of::<crate::sema::nested_outputs::Link>()
        .saturating_add(size_of::<crate::sema::nested_outputs::Formation>())
        .saturating_add(items.saturating_mul(
            size_of::<crate::sema::nested_outputs::Bridge>()
                + size_of::<crate::sema::closures::FormationInput>(),
        ));
    (bytes, items)
}

fn normalization_count(
    normalizations: &[s::NormalizationInput],
    work: &mut ImportedOriginals,
) -> Result<usize, &'static str> {
    let mut count = 0_usize;
    for proof in normalizations {
        work.charge(1)?;
        count = count
            .saturating_add(proof.sources.len())
            .saturating_add(proof.equalities.len())
            .saturating_add(proof.outlives.len())
            .saturating_add(proof.types.len())
            .saturating_add(1);
        if let Some(scoped) = &proof.scoped {
            count = count
                .saturating_add(scoped.scopes.len())
                .saturating_add(scoped.blocks.len())
                .saturating_add(1);
            for block in &scoped.blocks {
                work.charge(1)?;
                count = count
                    .saturating_add(block.equalities.len())
                    .saturating_add(block.outlives.len())
                    .saturating_add(block.types.len());
            }
        }
    }
    Ok(count)
}
