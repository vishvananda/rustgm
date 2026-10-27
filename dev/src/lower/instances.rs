//! Demand concrete CFGs from checked templates before ordinary executable lowering.
mod demand;
mod dependencies;
mod namespace_snapshot;
mod proof;
mod staged_originals;
#[cfg_attr(
    not(test),
    expect(
        unused_imports,
        reason = "private namespace transaction awaits qualified module-contract publication; public profile guards remain closed"
    )
)]
pub(crate) use namespace_snapshot::inspect_source_namespace;
#[cfg(test)]
#[path = "../../tests/invariants/module_snapshot_fixture.rs"]
mod namespace_fixtures;
#[cfg(test)]
pub(crate) use namespace_fixtures::namespace_fixture;
#[cfg(test)]
#[path = "../../tests/invariants/upstream_originals.rs"]
mod upstream_originals;
pub use super::transported_source::SourceInspection;
pub(crate) use super::transported_source::TraitInspection;
pub use crate::sema::crates::authored_export::SourceWork;
use crate::{
    body::{self, regions::Contracts},
    borrow,
    resolve::{expansion::Report, model::DefId, package::Error},
    sema::model::Declarations,
};

/// Inspect actual source-provider transport rows after complete original,
/// instance and assembly success. This publishes no metadata descriptor or
/// imported role/execution permission.
/// # Errors
/// Preserves unsupported source forms, incomplete coverage and every ordinary
/// proof/lowering failure before returning neutral observations.
pub fn inspect_source_transport(
    report: &Report,
    facts: &mut Declarations,
    identity: &crate::support::crates::CrateIdentity,
    prepared: crate::pipeline::Prepared,
) -> Result<SourceInspection, Error> {
    inspect_source_rows(report, facts, identity, prepared, false)
}

/// Inspect complete actual numbering rows as unpaid development data.
/// This entry cannot publish a descriptor or activate imported defaults.
/// # Errors
/// Preserves original proof failures and incomplete allocation/unit transport.
pub fn inspect_source_transport_with_numbering(
    report: &Report,
    facts: &mut Declarations,
    identity: &crate::support::crates::CrateIdentity,
    prepared: crate::pipeline::Prepared,
) -> Result<SourceInspection, Error> {
    inspect_source_rows(report, facts, identity, prepared, true)
}

fn inspect_source_rows(
    report: &Report,
    facts: &mut Declarations,
    identity: &crate::support::crates::CrateIdentity,
    prepared: crate::pipeline::Prepared,
    numbered: bool,
) -> Result<SourceInspection, Error> {
    collect_source(
        report,
        facts,
        prepared,
        |facts, checked| {
            if numbered {
                super::transported_source::Source::begin_numbered(report, facts, checked, identity)
            } else {
                super::transported_source::Source::begin(report, facts, checked, identity)
            }
            .map_err(|e| diagnostic(report, None, e.into()))
        },
        |source, _| {
            source
                .complete()
                .map_err(|e| diagnostic(report, None, e.into()))
        },
    )
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "private numbered trait inputs await semantic body domains and complete Provided original/instance proof; public guards stay closed"
    )
)]
pub(crate) fn inspect_source_trait_inputs_with_numbering(
    report: &Report,
    facts: &mut Declarations,
    identity: &crate::support::crates::CrateIdentity,
    prepared: crate::pipeline::Prepared,
) -> Result<TraitInspection, Error> {
    collect_source(
        report,
        facts,
        prepared,
        |facts, checked| {
            super::transported_source::Source::begin_trait_numbered(
                report, facts, checked, identity,
            )
            .map_err(|e| diagnostic(report, None, e.into()))
        },
        |source, facts| source.complete_trait_numbered(report, facts),
    )
}

fn collect_source<R>(
    report: &Report,
    facts: &mut Declarations,
    prepared: crate::pipeline::Prepared,
    begin: impl FnOnce(
        &mut Declarations,
        &borrow::templates::Checked<'_>,
    ) -> Result<super::transported_source::Source, Error>,
    complete: impl FnOnce(super::transported_source::Source, &mut Declarations) -> Result<R, Error>,
) -> Result<R, Error> {
    inspect_staged(
        report,
        facts,
        prepared,
        |facts, checked| {
            let source = begin(facts, checked)?;
            let exports = super::roots::exports(facts, &[])
                .map_err(|e| diagnostic(report, None, e.into()))?;
            Ok((None, exports, source))
        },
        |source, facts, proved, input| {
            source
                .collect(facts, proved, input)
                .map_err(|e| diagnostic(report, None, e.into()))
        },
        |facts, assembled, roots, source| {
            super::build_assembled(facts, assembled, roots)
                .map_err(|e| diagnostic(report, e.node, e.message.into()))?;
            complete(source, facts)
        },
        None,
    )?
}

/// Compile selected instances using retained syntax/type facts and checked templates.
/// Each demanded body has its own substituted contract; source keys are diagnostic
/// identities, never keys that combine different instances' lifetime contexts.
/// # Errors
/// Rejects invalid unused templates, unsupported demanded bodies, proof failures
/// and bounded instance growth.
pub fn build(
    report: &Report,
    facts: &mut Declarations,
    entry: DefId,
) -> Result<crate::lowir::program::Program, Error> {
    build_prepared(report, facts, entry, crate::pipeline::Prepared::default())
}

/// Consume defining CFGs and contracts already built for semantic dependencies.
/// # Errors
/// Applies the same complete template and instance gates as `build`.
pub fn build_prepared(
    report: &Report,
    facts: &mut Declarations,
    entry: DefId,
    prepared: crate::pipeline::Prepared,
) -> Result<crate::lowir::program::Program, Error> {
    build_roots_prepared(report, facts, super::Roots::executable(entry), prepared)
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the real selected consumer remains private until encoded Provided admission and complete original/instance execution qualify"
    )
)]
pub(crate) fn build_selected_prepared(
    report: &Report,
    facts: &mut Declarations,
    entry: DefId,
    prepared: crate::pipeline::Prepared,
    preparation: &mut dyn crate::sema::dispatch::Preparation,
) -> Result<crate::lowir::program::Program, Error> {
    if report.package.staged_consumer().is_none() {
        return Err(diagnostic(
            report,
            None,
            "selected lowering requires its actual private consumer".into(),
        ));
    }
    inspect_staged(
        report,
        facts,
        prepared,
        |facts, _| {
            let exports = super::roots::exports(facts, &[])
                .map_err(|e| diagnostic(report, None, e.into()))?;
            Ok((Some(entry), exports, ()))
        },
        staged_originals::ignore,
        |facts, assembled, roots, ()| super::build_assembled(facts, assembled, roots),
        Some(preparation),
    )?
    .map_err(|error| diagnostic(report, error.node, error.message.into()))
}
/// Check and lower explicit library roots alongside canonical registry exports.
/// # Errors
/// Applies the same template, ownership and instance gates as executable lowering.
pub fn build_roots_prepared(
    report: &Report,
    facts: &mut Declarations,
    roots: super::Roots<'_>,
    prepared: crate::pipeline::Prepared,
) -> Result<crate::lowir::program::Program, Error> {
    inspect_roots(report, facts, roots, prepared, |facts, assembled, roots| {
        super::build_assembled(facts, assembled, roots)
    })?
    .map_err(|e| diagnostic(report, e.node, e.message.into()))
}

/// Prove source originals before forming the library's transport interface and roots.
/// # Errors
/// Preserves source proof failures ahead of metadata profile restrictions; demanded
/// instances and lowering still complete before either artifact is returned.
pub fn build_library_prepared(
    report: &Report,
    facts: &mut Declarations,
    identity: crate::support::crates::CrateIdentity,
    prepared: crate::pipeline::Prepared,
) -> Result<
    (
        crate::lowir::program::Program,
        crate::support::crates::Interface,
    ),
    Error,
> {
    if report.package.source_core().is_some() {
        return build_source_library_profile(
            report,
            facts,
            identity,
            prepared,
            SourcePublication::LegacyFunctions,
        );
    }
    let mut interface = None;
    let program = inspect_selected(
        report,
        facts,
        prepared,
        |facts, checked| {
            let exported = crate::sema::crates::export_with_originals(
                report,
                facts,
                identity,
                |facts, requests, base| {
                    super::crate_originals::export(report, facts, checked, requests, base)
                },
            )?;
            let roots = super::roots::exports(facts, &exported.roots)
                .map_err(|e| diagnostic(report, None, e.into()))?;
            interface = Some(exported.interface);
            Ok((None, roots))
        },
        super::build_assembled,
    )?
    .map_err(|e| diagnostic(report, e.node, e.message.into()))?;
    let interface = interface
        .ok_or_else(|| diagnostic(report, None, "missing checked library exports".into()))?;
    Ok((program, interface))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourcePublication {
    LegacyFunctions,
    FunctionContracts,
    ModuleContracts,
    ProvidedContracts,
}

/// Publish only an explicitly selected ordinary source row profile.
/// # Errors
/// Retains actual source proof, contract and artifact eligibility failures.
pub fn build_source_library_profile(
    report: &Report,
    facts: &mut Declarations,
    identity: crate::support::crates::CrateIdentity,
    prepared: crate::pipeline::Prepared,
    profile: SourcePublication,
) -> Result<
    (
        crate::lowir::program::Program,
        crate::support::crates::Interface,
    ),
    Error,
> {
    if report.package.source_core().is_none() {
        return Err(diagnostic(
            report,
            None,
            "source publication profile requires its actual SourceCore provider".into(),
        ));
    }
    inspect_staged(
        report,
        facts,
        prepared,
        |facts, checked| {
            let (source, modules) = super::transported_source::Source::begin_publication(
                report, facts, checked, &identity, profile,
            )
            .map_err(|e| diagnostic(report, None, e.into()))?;
            let mut dependencies =
                report
                    .resolution
                    .crates
                    .as_ref()
                    .map_or_else(Vec::new, |imported| {
                        imported
                            .dependencies()
                            .aliases()
                            .iter()
                            .map(|alias| alias.identity().clone())
                            .collect::<Vec<_>>()
                    });
            dependencies.sort_unstable();
            dependencies.dedup();
            let base =
                crate::support::crates::Interface::new(identity.clone(), dependencies, Vec::new())
                    .map_err(|e| diagnostic(report, None, e.into()))?;
            facts
                .crates
                .get_or_insert_default()
                .set_identity(identity)
                .map_err(|e| diagnostic(report, None, e.into()))?;
            let roots = super::roots::exports(facts, &[])
                .map_err(|e| diagnostic(report, None, e.into()))?;
            Ok((None, roots, (source, base, modules)))
        },
        |(source, _, _), facts, proved, input| {
            source
                .collect(facts, proved, input)
                .map_err(|e| diagnostic(report, None, e.into()))
        },
        |facts, assembled, roots, (source, base, modules)| {
            let program = super::build_assembled(facts, assembled, roots)
                .map_err(|e| diagnostic(report, e.node, e.message.into()))?;
            let interface = match profile {
                SourcePublication::LegacyFunctions => source.complete_payload(report, facts, base),
                SourcePublication::FunctionContracts => {
                    source.complete_contract_payload(report, facts, base)
                }
                SourcePublication::ProvidedContracts => {
                    source.complete_provided_contract_payload(report, facts, base)
                }
                SourcePublication::ModuleContracts => source.complete_module_contract_payload(
                    modules.ok_or_else(|| {
                        diagnostic(
                            report,
                            None,
                            "module namespace/domain collection is absent".into(),
                        )
                    })?,
                    report,
                    facts,
                    base,
                ),
            }
            .map_err(|error| facts.retain_error_warnings(error))?;
            Ok((program, interface))
        },
        None,
    )?
}

/// Inspect the complete checked source program without requiring an executable entry.
/// The observer cannot retain the borrowed proof or start mutable semantic queries.
/// # Errors
/// Requires source-prepared initializer validation and applies the ordinary unused
/// body, template, instance, storage, ownership, loan and alignment checks.
pub fn inspect_prepared<R>(
    report: &Report,
    facts: &mut Declarations,
    prepared: crate::pipeline::Prepared,
    inspect: impl FnOnce(&Declarations, &borrow::Checked<'_>) -> R,
) -> Result<R, Error> {
    if prepared.constants.is_none() {
        return Err(diagnostic(
            report,
            None,
            "constant inspection requires source preparation".into(),
        ));
    }
    inspect_roots(
        report,
        facts,
        super::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
        |facts, assembled, _| {
            assembled.with_checked(|checked| {
                super::alignment::check(facts, checked.program())?;
                Ok(inspect(facts, checked))
            })
        },
    )?
    .map_err(|e: super::Error| diagnostic(report, e.node, e.message.into()))
}

fn inspect_roots<R>(
    report: &Report,
    facts: &mut Declarations,
    roots: super::Roots<'_>,
    prepared: crate::pipeline::Prepared,
    inspect: impl FnOnce(
        &mut Declarations,
        borrow::incremental::Assembled,
        super::roots::Selected<'_>,
    ) -> R,
) -> Result<R, Error> {
    inspect_selected(
        report,
        facts,
        prepared,
        |facts, _| {
            let exports = super::roots::exports(facts, roots.exports)
                .map_err(|e| diagnostic(report, None, e.into()))?;
            Ok((roots.entry, exports))
        },
        inspect,
    )
}

fn inspect_selected<R>(
    report: &Report,
    facts: &mut Declarations,
    prepared: crate::pipeline::Prepared,
    select: impl FnOnce(
        &mut Declarations,
        &borrow::templates::Checked<'_>,
    ) -> Result<(Option<DefId>, super::roots::Exports), Error>,
    inspect: impl FnOnce(
        &mut Declarations,
        borrow::incremental::Assembled,
        super::roots::Selected<'_>,
    ) -> R,
) -> Result<R, Error> {
    inspect_staged(
        report,
        facts,
        prepared,
        |facts, checked| select(facts, checked).map(|(entry, exports)| (entry, exports, ())),
        staged_originals::ignore,
        |facts, assembled, roots, ()| inspect(facts, assembled, roots),
        None,
    )
}

fn require_source_session(report: &Report, facts: &Declarations) -> Result<(), Error> {
    if let Some(catalog) = facts.source_bindings.as_deref() {
        catalog
            .require_provider_report(report)
            .map_err(|e| diagnostic(report, None, e.into()))?;
    } else if report.package.uses_source_bindings() {
        return Err(diagnostic(
            report,
            None,
            "source lowering requires its actual prepared catalog".into(),
        ));
    }
    Ok(())
}

fn inspect_staged<S, R>(
    report: &Report,
    facts: &mut Declarations,
    mut prepared: crate::pipeline::Prepared,
    select: impl FnOnce(
        &mut Declarations,
        &borrow::templates::Checked<'_>,
    ) -> Result<(Option<DefId>, super::roots::Exports, S), Error>,
    collect: impl FnMut(
        &mut S,
        &mut Declarations,
        &borrow::incremental::Proved,
        &body::regions::Contract,
    ) -> Result<(), Error>,
    inspect: impl FnOnce(
        &mut Declarations,
        borrow::incremental::Assembled,
        super::roots::Selected<'_>,
        S,
    ) -> R,
    preparation: Option<&mut dyn crate::sema::dispatch::Preparation>,
) -> Result<R, Error> {
    let (templates, contexts, inputs) = prepare_template_inputs(report, facts, &mut prepared)?;
    let templates = std::rc::Rc::new(templates);
    if let Some(context) = &prepared.constants {
        context
            .templates(std::rc::Rc::clone(&templates))
            .map_err(|e| diagnostic(report, None, e.into()))?;
    }

    let contracts = Contracts {
        bodies: inputs,
        callables: &facts.lifetimes.callables,
        nominal_requirements: &facts.lifetimes.nominal_requirements,
    };
    let custom = facts
        .destructors
        .implementations()
        .iter()
        .map(|i| i.item)
        .collect::<Vec<_>>();
    let mut copies = crate::types::copy::Copies::default();
    let mut environment = borrow::Environment {
        types: &facts.types,
        definitions: &facts.aggregates,
        derives: &facts.derives,
        source_copy: facts.uses_source_copy(),
        copies: &mut copies,
    };
    let checked = borrow::templates::verify_with_context(
        &mut environment,
        &facts.lifetimes.variances,
        &contracts,
        &templates,
        &contexts,
        &custom,
        |ty, field| body::drops::nominal_position(facts, ty, field),
    )
    .map_err(|e| diagnostic(report, e.node, e.message))?;
    let inputs = contracts.bodies;
    publish_closure_conditions(facts, &checked).map_err(|e| diagnostic(report, None, e.into()))?;
    publish_source_opaque_heads(facts, &checked)
        .map_err(|(source, message)| diagnostic(report, Some(source.into()), message.into()))?;
    facts
        .prepare_opaque_storage_located()
        .map_err(|failure| opaque_storage_error(report, facts, failure))?;
    let (entry, exports, mut stage) = select(facts, &checked)?;
    let roots = exports.selected(entry);
    let mut collect = staged_originals::Binding {
        stage: &mut stage,
        callback: collect,
    };
    let mut originals = staged_originals::Context {
        checked: &checked,
        collector: &mut collect,
        preparation,
    };
    #[cfg(test)]
    eprintln!(
        "selected-dispatch context={}",
        std::mem::size_of_val(&originals)
    );
    let program = if originals.preparation.is_some() {
        let (program, ordinary) = demand::prepare_ordinary(
            report,
            facts,
            &mut originals,
            inputs,
            prepared.concrete,
            prepared.constants,
            roots.globals,
        )?;
        let service = originals.preparation.take().ok_or_else(|| {
            diagnostic(report, None, "selected original service disappeared".into())
        })?;
        let retained = checked
            .retain(std::rc::Rc::clone(&templates))
            .map_err(|e| diagnostic(report, e.node, e.message))?;
        // Proof outputs own every prepared context needed by later instances.
        drop(contexts);
        drop(templates);
        demand::materialize_selected(report, facts, roots, retained, program, &ordinary, service)?
    } else {
        demand::prepare(
            report,
            facts,
            roots,
            originals,
            inputs,
            prepared.concrete,
            prepared.constants,
        )?
    };
    let assembled = program
        .assemble()
        .map_err(|e| diagnostic(report, e.node, e.message))?;
    Ok(inspect(facts, assembled, roots, stage))
}

fn prepare_template_inputs(
    report: &Report,
    facts: &mut Declarations,
    prepared: &mut crate::pipeline::Prepared,
) -> Result<
    (
        body::templates::Templates,
        borrow::templates::Contexts,
        Vec<body::regions::Contract>,
    ),
    Error,
> {
    require_source_session(report, facts)?;
    let mut templates = std::mem::take(&mut prepared.templates).finish(report, facts)?;
    templates
        .attach(prepared.imported.take())
        .map_err(|e| diagnostic(report, None, e.into()))?;
    let mut inputs = if let Some(collector) = prepared.contracts.take() {
        collector.finish(facts)
    } else {
        body::regions::collect(facts)
    }
    .map_err(|e| diagnostic(report, None, e.into()))?
    .bodies;
    let mut contexts = if let Some(originals) = prepared.original_contexts.take() {
        originals
            .apply(&templates, &mut inputs)
            .map_err(|e| diagnostic(report, None, e.into()))?
    } else {
        borrow::templates::Contexts::default()
    };
    body::templates::constructors(report, facts, &mut templates, &mut inputs)?;
    body::provided::append(report, facts, &mut templates, &mut inputs)?;
    body::iterator_factories::append(report, facts, &mut templates, &mut inputs)?;
    body::promote::analyze_templates_reusing(
        report,
        facts,
        &mut templates,
        &mut prepared.promotions,
    )?;
    prepare_original_contracts(report, facts, &templates, &mut contexts, &mut inputs)?;
    Ok((templates, contexts, inputs))
}

fn publish_closure_conditions(
    facts: &mut Declarations,
    checked: &borrow::templates::Checked<'_>,
) -> Result<(), &'static str> {
    let condition_steps = checked.visit_closure_conditions(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
        |identity, source, conditions| {
            facts
                .closures
                .retain_conditions(identity, source, conditions)
        },
    )?;
    facts.body_work.canonical_steps += condition_steps;
    Ok(())
}

fn publish_source_opaque_heads(
    facts: &mut Declarations,
    checked: &borrow::templates::Checked<'_>,
) -> Result<(), (crate::body::ir::BodyOrigin, &'static str)> {
    if facts.opaques.owners.is_empty() {
        return Ok(());
    }
    let owners: crate::support::hash::HashMap<_, _> = facts
        .functions
        .iter()
        .filter(|f| facts.opaques.owners.contains_key(&f.definition))
        .filter_map(|f| f.body.map(|body| (body, f.definition)))
        .collect();
    for proof in &checked.regions().bodies {
        if let Some(source) = proof.source.source()
            && let Some(&owner) = owners.get(&source)
        {
            facts
                .close_opaque_results(owner, &proof.hidden_regions)
                .map_err(|error| (proof.source, error))?;
        }
    }
    Ok(())
}

fn opaque_storage_error(
    report: &Report,
    facts: &Declarations,
    failure: crate::sema::opaque::StorageFailure,
) -> Error {
    let source = failure.owner.and_then(|owner| {
        let function = facts
            .function_indices
            .get(&owner)
            .and_then(|&index| facts.functions.get(index))?;
        let declaration = report.resolution.definitions.get(owner.0 as usize)?;
        (function.definition == owner
            && declaration.kind == crate::resolve::model::DefKind::Function
            && declaration.node == Some(function.node))
        .then_some(function.node.into())
    });
    diagnostic(report, source, failure.message.into())
}

fn diagnostic(report: &Report, node: Option<crate::body::ir::ProofSite>, message: String) -> Error {
    report.diagnostic(report.body_optional_error(node, message))
}

fn prepare_original_contracts(
    report: &Report,
    facts: &mut Declarations,
    templates: &body::templates::Templates,
    contexts: &mut borrow::templates::Contexts,
    inputs: &mut [body::regions::Contract],
) -> Result<(), Error> {
    if facts.uses_source_copy()
        && let Some(originals) = templates.imported_originals()
    {
        if contexts.imported.is_some() {
            return Err(diagnostic(
                report,
                None,
                "imported proof inputs were prepared twice".into(),
            ));
        }
        contexts.imported = Some(Box::new(
            body::copy_requirements::ImportedInputs::build(
                facts,
                std::rc::Rc::clone(originals),
                crate::pipeline::OriginalInput::prepare_imported,
            )
            .map_err(|e| diagnostic(report, None, e))?,
        ));
    }
    if facts.uses_source_copy() {
        // The shared input inventory also contains concrete bodies built later.
        // Only actual original rows enter this proof; each concrete body must
        // independently prepare its carrier at dispatch before it can execute.
        let originals: crate::support::hash::HashSet<_> = templates
            .bodies
            .iter()
            .chain(&contexts.bodies.bodies)
            .map(|template| template.body.key())
            .collect();
        body::copy_requirements::originals(
            facts,
            templates
                .bodies
                .iter()
                .chain(&contexts.bodies.bodies)
                .map(|template| &template.body),
            inputs
                .iter_mut()
                .chain(&mut contexts.source)
                .filter(|contract| originals.contains(&contract.key)),
        )
        .map_err(|e| diagnostic(report, None, e))?;
    }
    for input in inputs {
        input
            .prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)
            .map_err(|e| diagnostic(report, Some(input.source.into()), e.into()))?;
    }
    Ok(())
}
