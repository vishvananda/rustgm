//! Iterative depth-first demand distinguishes recursion from unbounded instance growth.
use super::{dependencies::Dependencies, diagnostic, proof};
use crate::{
    body::{
        self,
        ir::{Body, BodyKey, NodeKey, Owner},
        regions::Contract,
    },
    borrow::templates::Checked,
    resolve::{expansion::Report, model::DefId, package::Error},
    sema::model::Declarations,
    types::{ItemId, Kind, instance::Instance},
};
use std::collections::{HashMap, HashSet};

enum Work {
    Enter(Instance, NodeKey),
    Leave(BodyKey),
}
#[derive(Default)]
struct Ordinary {
    functions: HashMap<ItemId, usize>,
    closures: HashMap<crate::types::closure::ClosureId, usize>,
}
#[derive(Default)]
struct Budget {
    active: HashMap<BodyKey, usize>,
    bytes: usize,
    instances: usize,
}
impl Budget {
    fn enter(&mut self, item: BodyKey, body: &Body, input: &Contract) -> Result<(), &'static str> {
        let active = self.active.entry(item).or_default();
        if *active >= 128 {
            return Err("recursive generic instantiation exceeds 128 active instances");
        }
        self.instances += 1;
        if self.instances > 100_000 {
            return Err("generic instantiation exceeds 100000 bodies");
        }
        let bytes = body
            .retained_bytes()
            .saturating_add(std::mem::size_of_val(input.obligations.as_slice()))
            .saturating_add(std::mem::size_of_val(input.assumptions.as_slice()))
            .saturating_add(std::mem::size_of_val(input.type_assumptions.as_slice()));
        self.bytes = self.bytes.saturating_add(bytes);
        if self.bytes > 64_000_000 {
            return Err("generic instance bodies exceed 64 MB budget");
        }
        *active += 1;
        Ok(())
    }
}
pub(super) fn prepare(
    report: &Report,
    facts: &mut Declarations,
    roots: crate::lower::Roots<'_>,
    templates: &Checked<'_>,
    inputs: Vec<Contract>,
    concrete: HashMap<DefId, Body>,
    constants: Option<crate::pipeline::execution::Context>,
) -> Result<proof::Builder, Error> {
    let (mut program, ordinary) =
        prepare_ordinary(report, facts, templates, inputs, concrete, constants)?;
    // Discovery includes implicit constructor factories. With no templates,
    // ordinary lowering already follows calls and selected cleanup dependencies.
    // All source bodies have still been checked before taking this path.
    if templates.is_empty() && !facts.types.has_dynamic_shapes() {
        return Ok(program);
    }
    let mut work = initial_work(report, facts, roots, &program, &ordinary)?;
    let mut instances = HashSet::new();
    let mut visited = HashSet::new();
    let mut dependencies = Dependencies::default();
    let mut budget = Budget::default();
    let mut cleanup = body::instantiate::Cleanup::new(facts);
    while let Some(action) = work.pop() {
        let (instance, source) = match action {
            Work::Enter(instance, source) => (instance, source),
            Work::Leave(item) => {
                let active = budget.active.get_mut(&item).ok_or_else(|| {
                    diagnostic(report, None, "invalid active instance stack".into())
                })?;
                *active = active
                    .checked_sub(1)
                    .ok_or_else(|| diagnostic(report, None, "unbalanced instance stack".into()))?;
                continue;
            }
        };
        let error = |e: &str| diagnostic(report, Some(source), e.into());
        let (family, selected, ordinary_index) = match facts.types.kind(instance.ty()) {
            Some(Kind::FunctionItem { item, .. }) => {
                if facts
                    .foreign
                    .as_deref()
                    .and_then(|r| r.function(DefId(item.0)))
                    .is_some_and(|f| f.linkage == crate::support::linkage::Linkage::Import)
                {
                    continue;
                }
                let selected = templates
                    .get(item)
                    .map(|template| {
                        templates
                            .input(item)
                            .map(|input| (template, input))
                            .ok_or_else(|| error("missing checked template input"))
                    })
                    .transpose()?;
                (
                    BodyKey::Declaration(DefId(item.0)),
                    selected,
                    ordinary.functions.get(&item).copied(),
                )
            }
            Some(Kind::Closure { identity, .. }) => (
                BodyKey::Closure(identity),
                templates.closure(identity),
                ordinary.closures.get(&identity).copied(),
            ),
            _ => return Err(error("invalid demanded instance")),
        };
        let index = if let Some((template, input)) = selected {
            if !instances.insert(instance) {
                continue;
            }
            budget.enter(family, &template.body, input).map_err(error)?;
            let mut materialized =
                // Entered dependencies come only from a body sealed by Builder::push.
                body::instantiate::from_checked_caller(facts, template, instance, input, &mut cleanup)
                    .map_err(error)?;
            body::dispatch::resolve(facts, &mut materialized.body, &mut materialized.contract)
                .map_err(|e| error(&e))?;
            facts.prepare_opaque_storage().map_err(error)?;
            body::promote::instance(
                report,
                facts,
                &mut materialized.body,
                &materialized.destruction,
            )?;
            body::constants::instance(report, facts, &materialized.body)?;
            work.push(Work::Leave(family));
            program
                .push(
                    facts,
                    templates,
                    materialized.body,
                    &materialized.contract,
                    Some(materialized.destruction),
                )
                .map_err(|e| diagnostic(report, e.node, e.message))?
        } else {
            let index = ordinary_index.ok_or_else(|| {
                error("demanded callable has no concrete body or checked template")
            })?;
            if !visited.insert(index) {
                continue;
            }
            index
        };
        dependencies.schedule(report, facts, &mut program, index, &mut work)?;
    }
    Ok(program)
}

fn initial_work(
    report: &Report,
    facts: &Declarations,
    roots: crate::lower::Roots<'_>,
    program: &proof::Builder,
    ordinary: &Ordinary,
) -> Result<Vec<Work>, Error> {
    let mut work = Vec::new();
    for definition in roots.iter().rev() {
        if facts
            .foreign
            .as_deref()
            .and_then(|registry| registry.function(definition))
            .is_some_and(|function| function.linkage == crate::support::linkage::Linkage::Import)
        {
            continue;
        }
        let source = ordinary
            .functions
            .get(&ItemId(definition.0))
            .and_then(|&i| program.body(i).ok().map(|body| body.source))
            .ok_or_else(|| diagnostic(report, None, "missing root body".into()))?;
        let ty = facts
            .definitions
            .get(definition.0 as usize)
            .copied()
            .flatten()
            .ok_or_else(|| diagnostic(report, Some(source), "missing root type".into()))?;
        let instance = Instance::new(&facts.types, ty)
            .map_err(|e| diagnostic(report, Some(source), e.into()))?;
        work.push(Work::Enter(instance, source));
    }
    Ok(work)
}

impl Dependencies {
    fn schedule(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        program: &mut proof::Builder,
        index: usize,
        work: &mut Vec<Work>,
    ) -> Result<(), Error> {
        let body = program
            .body(index)
            .map_err(|e| diagnostic(report, None, e.into()))?;
        let source = body.source;
        let error = |e: &str| diagnostic(report, Some(source), e.into());
        let demands = self.collect(report, facts, body).map_err(|e| error(&e))?;
        for ty in demands.closures {
            let instance = Instance::closure(&facts.types, ty).map_err(error)?;
            work.push(Work::Enter(instance, source));
        }
        let mut next = demands.calls;
        next.extend(
            program
                .object_dependencies(facts, &demands.object_types)
                .map_err(error)?,
        );
        next.extend(program.cleanup_dependencies(facts, index).map_err(error)?);
        work.extend(next.into_iter().rev().map(|i| Work::Enter(i, source)));
        Ok(())
    }
}

fn prepare_ordinary(
    report: &Report,
    facts: &mut Declarations,
    templates: &Checked<'_>,
    inputs: Vec<Contract>,
    concrete: HashMap<DefId, Body>,
    constants: Option<crate::pipeline::execution::Context>,
) -> Result<(proof::Builder, Ordinary), Error> {
    let mut program = body::concrete_prepared(
        report,
        facts,
        |owner| templates.get(ItemId(owner.0)).is_some(),
        concrete,
    )?;
    let mut inputs: HashMap<_, _> = inputs.into_iter().map(|c| (c.key, c)).collect();
    for body in &mut program.bodies {
        let input = inputs.get_mut(&body.key()).ok_or_else(|| {
            diagnostic(
                report,
                Some(body.source),
                "missing ordinary dispatch contract".into(),
            )
        })?;
        body::dispatch::resolve(facts, body, input)
            .map_err(|e| diagnostic(report, Some(body.source), e))?;
    }
    body::promote::analyze(report, facts, &mut program)?;
    if let Some(context) = constants {
        context.constants(report, facts, &mut program)?;
    } else {
        body::constants::analyze(report, facts, &program)?;
    }
    let (mut program, bodies) =
        proof::Builder::new(program).map_err(|e| diagnostic(report, None, e.into()))?;
    let mut ordinary = Ordinary::default();
    for body in bodies {
        let source = body.source;
        let closure = if let Owner::Closure { identity, .. } = body.owner {
            Some(identity)
        } else {
            None
        };
        let definition = if let Owner::Function { definition, .. } = body.owner {
            Some(definition)
        } else {
            None
        };
        let input = inputs.remove(&body.key()).ok_or_else(|| {
            diagnostic(
                report,
                Some(source),
                "missing ordinary body contract".into(),
            )
        })?;
        let index = program
            .push(facts, templates, body, &input, None)
            .map_err(|e| diagnostic(report, e.node, e.message))?;
        if let Some(definition) = definition {
            ordinary.functions.insert(ItemId(definition.0), index);
        }
        if let Some(identity) = closure {
            ordinary.closures.insert(identity, index);
        }
    }
    Ok((program, ordinary))
}
