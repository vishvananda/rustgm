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
/// A source family selects either its checked template or existing concrete body.
struct Selected<'a> {
    family: BodyKey,
    template: Option<(&'a body::templates::Template, &'a Contract)>,
    ordinary: Option<usize>,
}
impl Ordinary {
    fn select<'a>(
        &self,
        facts: &Declarations,
        templates: &'a Checked<'_>,
        instance: Instance,
    ) -> Result<Option<Selected<'a>>, &'static str> {
        Ok(Some(match facts.types.kind(instance.ty()) {
            Some(Kind::FunctionItem { item, .. }) => {
                if facts
                    .foreign
                    .as_deref()
                    .and_then(|r| r.function(DefId(item.0)))
                    .is_some_and(|f| f.linkage == crate::support::linkage::Linkage::Import)
                {
                    return Ok(None);
                }
                let template = templates
                    .get(item)
                    .map(|template| {
                        templates
                            .input(item)
                            .map(|input| (template, input))
                            .ok_or("missing checked template input")
                    })
                    .transpose()?;
                Selected {
                    family: BodyKey::Declaration(DefId(item.0)),
                    template,
                    ordinary: self.functions.get(&item).copied(),
                }
            }
            Some(Kind::Closure { identity, .. }) => Selected {
                family: BodyKey::Closure(identity),
                template: templates.closure(identity),
                ordinary: self.closures.get(&identity).copied(),
            },
            _ => return Err("invalid demanded instance"),
        }))
    }
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
    roots: crate::lower::roots::Selected<'_>,
    templates: &Checked<'_>,
    inputs: Vec<Contract>,
    concrete: HashMap<DefId, Body>,
    constants: Option<crate::pipeline::execution::Context>,
) -> Result<proof::Builder, Error> {
    let (mut program, ordinary) = prepare_ordinary(
        report,
        facts,
        templates,
        inputs,
        concrete,
        constants,
        roots.globals,
    )?;
    // Discovery includes implicit constructor factories. With no templates,
    // ordinary lowering already follows calls and selected cleanup dependencies.
    // All source bodies have still been checked before taking this path.
    if templates.is_empty() && !facts.types.has_dynamic_shapes() {
        return Ok(program);
    }
    let (mut work, mut dependencies) = initial_work(report, facts, roots, &mut program, &ordinary)?;
    let mut instances = HashSet::new();
    let mut visited = HashSet::new();
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
        let Some(Selected {
            family,
            template: selected,
            ordinary: ordinary_index,
        }) = ordinary.select(facts, templates, instance).map_err(error)?
        else {
            continue;
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
        dependencies.schedule(
            report,
            facts,
            &mut program,
            DependencyRoot::Body(index),
            &mut work,
        )?;
    }
    Ok(program)
}

fn initial_work(
    report: &Report,
    facts: &mut Declarations,
    roots: crate::lower::roots::Selected<'_>,
    program: &mut proof::Builder,
    ordinary: &Ordinary,
) -> Result<(Vec<Work>, Dependencies), Error> {
    let mut work = Vec::new();
    for definition in roots.functions.iter().rev() {
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
    let dependencies = Dependencies::roots(report, facts, program, roots.globals, &mut work)?;
    Ok((work, dependencies))
}

#[derive(Clone, Copy)]
enum DependencyRoot {
    Body(usize),
    Global(DefId),
}
impl Dependencies {
    fn roots(
        report: &Report,
        facts: &mut Declarations,
        program: &mut proof::Builder,
        globals: &[DefId],
        work: &mut Vec<Work>,
    ) -> Result<Self, Error> {
        let mut dependencies = Self::default();
        for &definition in globals {
            dependencies.schedule(
                report,
                facts,
                program,
                DependencyRoot::Global(definition),
                work,
            )?;
        }
        Ok(dependencies)
    }
    fn schedule(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        program: &mut proof::Builder,
        root: DependencyRoot,
        work: &mut Vec<Work>,
    ) -> Result<(), Error> {
        let (source, demands, cleanup) = match root {
            DependencyRoot::Body(index) => {
                let body = program
                    .body(index)
                    .map_err(|e| diagnostic(report, None, e.into()))?;
                let source = body.source;
                let demands = self
                    .collect(report, facts, body)
                    .map_err(|e| diagnostic(report, Some(source), e))?;
                let cleanup = program
                    .cleanup_dependencies(facts, index)
                    .map_err(|e| diagnostic(report, Some(source), e.into()))?;
                (source, demands, cleanup)
            }
            DependencyRoot::Global(definition) => {
                let initializer = facts
                    .initializer_index(definition)
                    .and_then(|index| facts.initializers.get(index))
                    .ok_or_else(|| {
                        diagnostic(
                            report,
                            None,
                            "global export requires a local initializer".into(),
                        )
                    })?;
                let source = initializer.value;
                let demands = self
                    .global(report, facts, definition)
                    .map_err(|e| diagnostic(report, Some(source), e))?;
                (source, demands, Vec::new())
            }
        };
        let error = |e: &str| diagnostic(report, Some(source), e.into());
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
        next.extend(cleanup);
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
    globals: &[DefId],
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
        context.constants(report, facts, &mut program, globals)?;
    } else {
        body::constants::analyze_bodies(report, facts, program.bodies.iter(), globals)?;
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
