//! Demand concrete CFGs from checked templates before ordinary executable lowering.
mod demand;
mod dependencies;
mod proof;
use crate::{
    body::{self, ir::NodeKey, regions::Contracts},
    borrow,
    resolve::{expansion::Report, model::DefId, package::Error},
    sema::model::Declarations,
};

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
/// Check and lower explicit library roots alongside canonical registry exports.
/// # Errors
/// Applies the same template, ownership and instance gates as executable lowering.
pub fn build_roots_prepared(
    report: &Report,
    facts: &mut Declarations,
    roots: super::Roots<'_>,
    prepared: crate::pipeline::Prepared,
) -> Result<crate::lowir::program::Program, Error> {
    inspect_roots(report, facts, roots, prepared, |facts, checked, roots| {
        super::build_exact_roots(facts, checked, roots)
    })?
    .map_err(|e| diagnostic(report, Some(e.node), e.message.into()))
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
        |facts, checked, _| {
            super::alignment::check(facts, checked.program())?;
            Ok(inspect(facts, checked))
        },
    )?
    .map_err(|e: super::Error| diagnostic(report, Some(e.node), e.message.into()))
}

fn inspect_roots<R>(
    report: &Report,
    facts: &mut Declarations,
    roots: super::Roots<'_>,
    mut prepared: crate::pipeline::Prepared,
    inspect: impl FnOnce(&mut Declarations, &borrow::Checked<'_>, super::roots::Selected<'_>) -> R,
) -> Result<R, Error> {
    let exports = super::roots::exports(facts, roots.exports)
        .map_err(|e| diagnostic(report, None, e.into()))?;
    let roots = exports.selected(roots.entry);
    let mut templates = prepared.templates.finish(report, facts)?;
    let mut inputs = if let Some(collector) = prepared.contracts {
        collector.finish(facts)
    } else {
        body::regions::collect(facts)
    }
    .map_err(|e| diagnostic(report, None, e.into()))?
    .bodies;
    body::templates::constructors(report, facts, &mut templates, &mut inputs)?;
    body::promote::analyze_templates_reusing(
        report,
        facts,
        &mut templates,
        &mut prepared.promotions,
    )?;
    let templates = std::rc::Rc::new(templates);
    if let Some(context) = &prepared.constants {
        context
            .templates(std::rc::Rc::clone(&templates))
            .map_err(|e| diagnostic(report, None, e.into()))?;
    }
    for input in &inputs {
        input
            .prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)
            .map_err(|e| diagnostic(report, Some(input.source), e.into()))?;
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
        copies: &mut copies,
    };
    let checked = borrow::templates::verify(
        &mut environment,
        &facts.lifetimes.variances,
        &contracts,
        &templates,
        &custom,
        |ty, field| body::drops::nominal_position(facts, ty, field),
    )
    .map_err(|e| diagnostic(report, e.node, e.message))?;
    let inputs = contracts.bodies;
    if !facts.opaques.owners.is_empty() {
        let owners: std::collections::HashMap<_, _> = facts
            .functions
            .iter()
            .filter(|f| facts.opaques.owners.contains_key(&f.definition))
            .filter_map(|f| f.body.map(|body| (body, f.definition)))
            .collect();
        for proof in &checked.regions().bodies {
            if let Some(&owner) = owners.get(&proof.source) {
                facts
                    .close_opaque_results(owner, &proof.hidden_regions)
                    .map_err(|e| diagnostic(report, Some(proof.source), e.into()))?;
            }
        }
    }
    facts
        .prepare_opaque_storage()
        .map_err(|e| diagnostic(report, None, e.into()))?;
    let program = demand::prepare(
        report,
        facts,
        roots,
        &checked,
        inputs,
        prepared.concrete,
        prepared.constants,
    )?;
    program
        .with_checked(|checked| inspect(facts, checked, roots))
        .map_err(|e| diagnostic(report, e.node, e.message))
}

fn diagnostic(report: &Report, node: Option<NodeKey>, message: String) -> Error {
    if let Some(key) = node {
        report.diagnostic(Declarations::error(report, key, message))
    } else {
        Error::at(crate::resolve::unit::FileId(0), 0, message)
    }
}
