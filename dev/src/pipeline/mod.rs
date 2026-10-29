//! Coordinate defining-body proofs without reversing semantic phase dependencies.
pub(crate) mod execution;
mod marked_definitions;
mod original_contexts;
use crate::support::hash::HashMap;
use crate::{
    body::{self, regions::Collector, templates},
    borrow,
    resolve::{expansion::Report, model::DefId, package::Error},
    sema::{
        self,
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
    analyze_defining(report, facts, None)
}
fn analyze_defining(
    report: &Report,
    facts: Declarations,
    context: Option<&execution::Context>,
) -> Result<Analysis, Error> {
    let mut demand = Demand::default();
    let facts =
        sema::bodies::analyze_defining(report, facts, |report, facts, request| match request {
            sema::bodies::DefiningRequest::Hidden(id) => demand.close(report, facts, id, context),
            sema::bodies::DefiningRequest::ComptimeDefinition(definition) => {
                demand.complete_comptime(report, facts, definition, context)
            }
        })?;
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
    let mut analysis = analyze_defining(report, facts, Some(&context))?;
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
    fn close(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        id: OpaqueId,
        context: Option<&execution::Context>,
    ) -> Result<(), Error> {
        let declaration = facts.opaques.declarations.get(&id).ok_or_else(|| {
            Error::at(
                crate::resolve::unit::FileId(0),
                0,
                "missing demanded hidden result",
            )
        })?;
        let owner = declaration.owner;
        let source = declaration.source;
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
