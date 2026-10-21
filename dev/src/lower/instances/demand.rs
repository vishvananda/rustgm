//! Iterative depth-first demand distinguishes recursion from unbounded instance growth.
use super::{dependencies::Dependencies, diagnostic, proof};
use crate::{
    body::{
        self,
        ir::{Body, NodeKey, Owner},
        regions::Contract,
    },
    borrow::templates::Checked,
    resolve::{expansion::Report, model::DefId, package::Error},
    sema::model::Declarations,
    types::{ItemId, Kind, instance::Instance},
};
use std::collections::{HashMap, HashSet};

enum Work {
    Body(usize),
    Enter(Instance, NodeKey),
    Leave(ItemId),
}
#[derive(Default)]
struct Ordinary {
    functions: HashMap<ItemId, usize>,
    closures: HashMap<crate::types::closure::ClosureId, usize>,
}
#[derive(Default)]
struct Budget {
    active: HashMap<ItemId, usize>,
    bytes: usize,
    instances: usize,
}
impl Budget {
    fn enter(&mut self, item: ItemId, body: &Body, input: &Contract) -> Result<(), &'static str> {
        let active = self.active.entry(item).or_default();
        if *active >= 128 {
            return Err("recursive generic instantiation exceeds 128 active instances");
        }
        self.instances += 1;
        if self.instances > 100_000 {
            return Err("generic instantiation exceeds 100000 bodies");
        }
        let bytes = body_bytes(body)
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
    entry: DefId,
    templates: &Checked<'_>,
    inputs: Vec<Contract>,
) -> Result<proof::Builder, Error> {
    let (mut program, ordinary) = prepare_ordinary(report, facts, templates, inputs)?;
    // Discovery includes implicit constructor factories. With no templates,
    // ordinary lowering already follows calls and selected cleanup dependencies.
    // All source bodies have still been checked before taking this path.
    if templates.is_empty() && !facts.types.has_dynamic_shapes() {
        return Ok(program);
    }
    let source = ordinary
        .functions
        .get(&ItemId(entry.0))
        .and_then(|&i| program.body(i).ok().map(|body| body.source))
        .ok_or_else(|| diagnostic(report, None, "missing entry body".into()))?;
    let entry_type = facts
        .definitions
        .get(entry.0 as usize)
        .copied()
        .flatten()
        .ok_or_else(|| diagnostic(report, Some(source), "missing entry type".into()))?;
    let entry = Instance::new(&facts.types, entry_type)
        .map_err(|e| diagnostic(report, Some(source), e.into()))?;
    let mut work = vec![Work::Enter(entry, source)];
    let mut instances = HashSet::new();
    let mut visited = HashSet::new();
    let mut dependencies = Dependencies::default();
    let mut budget = Budget::default();
    let mut cleanup = body::instantiate::Cleanup::new(facts);
    while let Some(action) = work.pop() {
        let index = if let Work::Body(index) = action {
            if !visited.insert(index) {
                continue;
            }
            index
        } else {
            let Work::Enter(instance, source) = action else {
                if let Work::Leave(item) = action {
                    let active = budget.active.get_mut(&item).ok_or_else(|| {
                        diagnostic(report, None, "invalid active instance stack".into())
                    })?;
                    *active = active.checked_sub(1).ok_or_else(|| {
                        diagnostic(report, None, "unbalanced instance stack".into())
                    })?;
                }
                continue;
            };
            let error = |e: &str| diagnostic(report, Some(source), e.into());
            let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(instance.ty()) else {
                return Err(error("invalid demanded instance"));
            };
            if let Some(template) = templates.get(item) {
                if !instances.insert(instance) {
                    continue;
                }
                let input = templates
                    .input(item)
                    .ok_or_else(|| error("missing checked template input"))?;
                budget.enter(item, &template.body, input).map_err(error)?;
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
                work.push(Work::Leave(item));
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
                let &index = ordinary.functions.get(&item).ok_or_else(|| {
                    error("demanded callable has no concrete body or checked template")
                })?;
                if !visited.insert(index) {
                    continue;
                }
                index
            }
        };
        dependencies.schedule(report, facts, &mut program, index, &ordinary, &mut work)?;
    }
    Ok(program)
}

impl Dependencies {
    fn schedule(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        program: &mut proof::Builder,
        index: usize,
        ordinary: &Ordinary,
        work: &mut Vec<Work>,
    ) -> Result<(), Error> {
        let body = program
            .body(index)
            .map_err(|e| diagnostic(report, None, e.into()))?;
        let source = body.source;
        let error = |e: &str| diagnostic(report, Some(source), e.into());
        let demands = self.collect(report, facts, body).map_err(|e| error(&e))?;
        for identity in demands.closures {
            let &index = ordinary
                .closures
                .get(&identity)
                .ok_or_else(|| error("missing demanded closure body"))?;
            work.push(Work::Body(index));
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
) -> Result<(proof::Builder, Ordinary), Error> {
    let mut program = body::concrete_except(report, facts, |owner| {
        templates.get(ItemId(owner.0)).is_some()
    })?;
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
    body::constants::analyze(report, facts, &program)?;
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

fn body_bytes(body: &Body) -> usize {
    use std::mem::size_of_val;
    let mut bytes = size_of::<Body>();
    for part in [
        size_of_val(body.locals.as_slice()),
        size_of_val(body.values.as_slice()),
        size_of_val(body.projections.as_slice()),
        size_of_val(body.temporaries.as_slice()),
        size_of_val(body.guards.as_slice()),
        size_of_val(body.guard_places.as_slice()),
        size_of_val(body.scalars.as_slice()),
        size_of_val(body.arguments.as_slice()),
        size_of_val(body.statements.as_slice()),
        size_of_val(body.blocks.as_slice()),
    ] {
        bytes = bytes.saturating_add(part);
    }
    if let Some(storage) = &body.storage {
        bytes = bytes
            .saturating_add(size_of_val(storage.scopes.as_slice()))
            .saturating_add(size_of_val(storage.exits.as_slice()));
    }
    if let Some(owned) = &body.owned {
        bytes = bytes
            .saturating_add(size_of_val(owned.created.as_slice()))
            .saturating_add(size_of_val(owned.returned.as_slice()));
    }
    bytes
}
