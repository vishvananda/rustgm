//! Coordinate defining-body proofs without reversing semantic phase dependencies.
pub(crate) mod execution;
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

/// Definition CFGs and contracts already demanded during semantic analysis.
#[derive(Default)]
pub struct Prepared {
    pub(crate) constants: Option<execution::Context>,
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
    functions: Option<HashMap<DefId, Function>>,
    custom: Option<Vec<ItemId>>,
    copies: crate::types::copy::Copies,
}

/// Infer bodies and prove any defining CFG needed to expose captured hidden lifetimes.
/// Every demanded CFG and contract is retained for the ordinary lowering pipeline.
/// # Errors
/// Reports semantic, lifetime, ownership and loan failures at their original source.
pub fn analyze(report: &Report, facts: Declarations) -> Result<Analysis, Error> {
    let mut demand = Demand::default();
    let facts = sema::bodies::analyze_with_hidden(report, facts, |report, facts, id| {
        demand.close(report, facts, id)
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
    let mut analysis = analyze(report, facts)?;
    context.check_inline_bodies(report, &mut analysis.facts)?;
    if !analysis.facts.initializers.is_empty() {
        sema::values::Evaluator::new(report, &mut analysis.facts)
            .validate_ordinary_initializers()?;
    }
    context.transfer(report, &mut analysis.facts, &mut analysis.prepared)?;
    analysis.prepared.constants = Some(context);
    Ok(analysis)
}

impl Demand {
    fn close(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        id: OpaqueId,
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
        let functions = self
            .functions
            .get_or_insert_with(|| facts.functions.iter().map(|f| (f.definition, *f)).collect());
        let function = functions
            .get(&owner)
            .ok_or_else(|| report.body_error(source, "missing defining function"))?;
        let mut template = self.prepared.templates.function(report, facts, function)?;
        body::promote::instance_reusing(
            report,
            facts,
            &mut template.body,
            &template.destruction,
            &mut self.prepared.promotions,
        )?;
        let input = self
            .prepared
            .contracts
            .get_or_insert_with(Collector::default)
            .function(facts, function)
            .map_err(|e| report.body_error(source, e))?;
        let mut input = input.clone();
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
        let templates = templates::Templates {
            bodies: vec![template],
        };
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
            self.prepared
                .templates
                .retain(template)
                .map_err(|e| report.body_error(source, e))?;
        }
        Ok(())
    }
}
