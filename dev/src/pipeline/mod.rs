//! Coordinate defining-body proofs without reversing semantic phase dependencies.
mod asyncbody;
pub(crate) mod cdrop;
pub(crate) mod execution;
pub(crate) use asyncbody::checked_future_storage;
pub use asyncbody::future_application;
pub(crate) use asyncbody::future_constructor;
pub(crate) use asyncbody::future_drop;
pub(crate) use asyncbody::future_poll;
mod marked_definitions;
mod original_contexts;
use crate::support::hash::HashMap;
use crate::{
    body::{self, regions::Collector, templates},
    borrow,
    resolve::{expansion::Report, model::DefId, package::Error},
    sema::{
        self,
        census::{Census, Phase},
        model::{Declarations, Function},
    },
    types::{ItemId, opaque::OpaqueId},
};
pub(crate) use original_contexts::Original as OriginalInput;

/// Definition CFGs and contracts already demanded during semantic analysis.
#[derive(Default)]
pub struct Prepared {
    pub(crate) constants: Option<execution::Context>,
    pub(crate) imported: Option<std::rc::Rc<crate::body::transported_originals::ImportedOriginals>>,
    pub(crate) original_contexts: Option<Box<original_contexts::Originals>>,
    pub(crate) concrete: HashMap<body::ir::BodyKey, body::ir::Body>,
    pub(crate) templates: templates::Prepared,
    pub(crate) contracts: Option<Collector>,
    pub(crate) promotions: body::promote::Scratch,
}
#[derive(Clone, Copy, Debug)]
pub struct Work {
    pub templates: usize,
    pub lifetime_facts: usize,
    pub constant_preparation: usize,
}
impl Prepared {
    /// Actual defining CFG constructions, consumed facts and constant index setup.
    #[must_use]
    pub fn work(&self) -> Work {
        Work {
            templates: self.templates.work().0,
            lifetime_facts: self.contracts.as_ref().map_or(0, Collector::observations),
            constant_preparation: self.promotions.preparation_steps(),
        }
    }
}
pub struct Analysis {
    pub facts: Declarations,
    pub prepared: Prepared,
}
#[derive(Default)]
struct Demand {
    prepared: Prepared,
    custom: Option<Vec<ItemId>>,
    copies: crate::types::copy::Copies,
}

/// Infer bodies and prove any defining CFG needed to expose captured hidden lifetimes.
/// Every demanded CFG and contract is retained for the ordinary lowering pipeline.
/// # Errors
/// Reports semantic, lifetime, ownership and loan failures at their original source.
pub fn analyze(report: &Report, facts: Declarations) -> Result<Analysis, Error> {
    analyze_defining(report, facts, None, None)
}
fn analyze_defining(
    report: &Report,
    facts: Declarations,
    context: Option<&execution::Context>,
    census: Option<&mut Census>,
) -> Result<Analysis, Error> {
    let mut demand = Demand::default();
    let facts = sema::bodies::analyze_defining(
        report,
        facts,
        |report, facts, request| match request {
            sema::bodies::DefiningRequest::Hidden(id) => demand.close(report, facts, id, context),
            sema::bodies::DefiningRequest::ComptimeDefinition(definition) => {
                demand.complete_const_definition(report, facts, definition, context)
            }
            sema::bodies::DefiningRequest::ConstDefinition(definition) => {
                Demand::const_readiness(report, facts, definition)
            }
        },
        census,
    )?;
    Ok(Analysis {
        facts,
        prepared: demand.prepared,
    })
}
/// Resolve declarations and bodies with one retained checked constant provider.
/// # Errors
/// Preserves all semantic and demanded-constant diagnostics.
pub fn analyze_source(report: &Report) -> Result<Analysis, Error> {
    let context = execution::Context::default();
    let facts = sema::declarations::analyze_with_executor(report, context.executor())?;
    let analysis = analyze_defining(report, facts, Some(&context), None)?;
    complete_source(report, context, analysis)
}
/// Run the same source analysis as an analysis-only census.
///
/// Isolated declaration and body loops record every failure they reach; budget
/// exhaustion and failures outside those loops end the census. The analysis is
/// returned only when every pass completed without a record. It is diagnostic
/// evidence for the caller's census and must never be lowered or published.
#[must_use]
pub fn census_source(report: &Report) -> (Census, Option<Analysis>) {
    let mut census = Census::default();
    let context = execution::Context::default();
    let Some(facts) =
        sema::declarations::census_with_executor(report, context.executor(), &mut census)
    else {
        return (census, None);
    };
    let analysis = match analyze_defining(report, facts, Some(&context), Some(&mut census)) {
        Ok(analysis) => analysis,
        Err(error) => {
            census.stop(Phase::Bodies, &error);
            return (census, None);
        }
    };
    if census.has_failures(Phase::Bodies) {
        census.stop_after_failures(Phase::Bodies);
        return (census, None);
    }
    match complete_source(report, context, analysis) {
        Ok(analysis) => (census, Some(analysis)),
        Err(error) => {
            census.stop(Phase::Completion, &error);
            (census, None)
        }
    }
}
/// Complete constants, the panic handler and lints after every body is checked.
fn complete_source(
    report: &Report,
    context: execution::Context,
    mut analysis: Analysis,
) -> Result<Analysis, Error> {
    complete_constants(
        report,
        &context,
        &mut analysis.facts,
        &mut analysis.prepared,
    )?;
    sema::compiler_items::complete_panic_handler(report, &mut analysis.facts).map_err(|error| {
        analysis
            .facts
            .retain_error_warnings(report.diagnostic(error))
    })?;
    analysis
        .facts
        .finalize_lints(report, true)
        .map_err(|error| {
            analysis
                .facts
                .retain_error_warnings(report.diagnostic(error))
        })?;
    analysis.prepared.constants = Some(context);
    Ok(analysis)
}

/// Prepare explicit bodies through the ordinary declaration and constant services.
pub(crate) fn prepare_local_runtime(
    report: &Report,
    roots: &[DefId],
) -> Result<(Declarations, Prepared), Error> {
    let context = execution::Context::default();
    let mut facts =
        crate::sema::declarations::analyze_local_roots(report, roots, context.executor())?;
    let mut prepared = Prepared::default();
    complete_constants(report, &context, &mut facts, &mut prepared)?;
    sema::compiler_items::complete_panic_handler(report, &mut facts)
        .map_err(|error| facts.retain_error_warnings(report.diagnostic(error)))?;
    facts
        .finalize_lints(report, true)
        .map_err(|error| facts.retain_error_warnings(report.diagnostic(error)))?;
    prepared.constants = Some(context);
    Ok((facts, prepared))
}

fn complete_constants(
    report: &Report,
    context: &execution::Context,
    facts: &mut Declarations,
    prepared: &mut Prepared,
) -> Result<(), Error> {
    context
        .check_inline_bodies(report, facts)
        .map_err(|error| facts.retain_error_warnings(error))?;
    if !facts.initializers.is_empty() {
        let validated =
            sema::values::Evaluator::new(report, facts).validate_ordinary_initializers();
        validated.map_err(|error| facts.retain_error_warnings(error))?;
    }
    context
        .transfer(report, facts, prepared)
        .map_err(|error| facts.retain_error_warnings(error))
}

impl Demand {
    fn demanded_definition(
        report: &Report,
        facts: &Declarations,
        id: OpaqueId,
    ) -> Result<(DefId, crate::resolve::unit::NodeKey), Error> {
        let declaration = facts.opaques.declarations.get(&id).ok_or_else(|| {
            Error::at(
                crate::resolve::unit::FileId(0),
                0,
                "missing demanded hidden result",
            )
        })?;
        declaration
            .require_current(facts, id)
            .map_err(Error::unlocated)?;
        let source = declaration.source().map_err(|message| {
            declaration
                .site(facts)
                .map_or_else(Error::unlocated, |site| report.body_error(site, message))
        })?;
        Ok((declaration.owner, source))
    }
    fn close(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        id: OpaqueId,
        context: Option<&execution::Context>,
    ) -> Result<(), Error> {
        let (owner, source) = Self::demanded_definition(report, facts, id)?;
        let function = Self::defining_function(facts, owner)?;
        let Some(original) = self.defining_original(report, facts, &function, context)? else {
            return if facts
                .opaques
                .declarations
                .get(&id)
                .is_some_and(|d| d.hidden.is_some())
            {
                Ok(())
            } else {
                Err(report.body_error(
                    source,
                    "cached defining original did not close its hidden result",
                ))
            };
        };
        if matches!(
            original.template.body.owner,
            body::ir::Owner::AsyncBody { .. }
        ) {
            original
                .validate(facts)
                .map_err(|e| report.body_error(source, e))?;
        }
        let original_contexts::Original {
            mut template,
            contract: mut input,
        } = original;
        body::promote::instance_reusing(
            report,
            facts,
            &mut template.body,
            &template.destruction,
            &mut self.prepared.promotions,
        )?;
        body::copy_requirements::prepare(facts, &template.body, &mut input)
            .map_err(|e| report.body_error(source, e))?;
        input
            .prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)
            .map_err(|e| report.body_error(source, e))?;
        let contracts = body::regions::Contracts {
            bodies: vec![input.clone()],
            callables: &facts.lifetimes.callables,
            nominal_requirements: &facts.lifetimes.nominal_requirements,
        };
        let templates = templates::Templates::source(vec![template]);
        let custom = self.custom.get_or_insert_with(|| {
            facts
                .destructors
                .implementations()
                .iter()
                .map(|i| i.item)
                .collect()
        });
        let checked = borrow::templates::verify(
            &mut borrow::Environment {
                types: &facts.types,
                definitions: &facts.aggregates,
                derives: &facts.derives,
                source_copy: facts.uses_source_copy(),
                copies: &mut self.copies,
            },
            &facts.lifetimes.variances,
            &contracts,
            &templates,
            custom,
            |ty, field| body::drops::nominal_position(facts, ty, field),
        )
        .map_err(|e| report.body_error(e.node.unwrap_or(source.into()), e.message))?;
        let proof = checked
            .regions()
            .bodies
            .first()
            .ok_or_else(|| report.body_error(source, "missing defining region proof"))?;
        let choices = proof.hidden_regions.clone();
        if matches!(proof.key, body::ir::BodyKey::AsyncBody(_)) {
            let arena = checked
                .future_store(&facts.types)
                .map_err(|e| report.body_error(source, e))?;
            facts
                .infer_checked_async_future(owner, arena)
                .map_err(|e| report.body_error(source, e))?;
        }
        drop(checked);
        facts
            .close_opaque_results(owner, &choices)
            .map_err(|e| report.body_error(source, e))?;
        for template in templates.bodies {
            self.retain_defining_original(
                report,
                facts,
                original_contexts::Original {
                    template,
                    contract: input.clone(),
                },
                context,
            )?;
        }
        Ok(())
    }
}
