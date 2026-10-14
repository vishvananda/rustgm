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
    Enter(Instance, NodeKey),
    Leave(ItemId),
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
    if templates.is_empty() {
        return Ok(program);
    }
    let source = ordinary
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
    while let Some(action) = work.pop() {
        let Work::Enter(instance, source) = action else {
            if let Work::Leave(item) = action {
                let active = budget.active.get_mut(&item).ok_or_else(|| {
                    diagnostic(report, None, "invalid active instance stack".into())
                })?;
                *active = active
                    .checked_sub(1)
                    .ok_or_else(|| diagnostic(report, None, "unbalanced instance stack".into()))?;
            }
            continue;
        };
        let error = |e: &str| diagnostic(report, Some(source), e.into());
        let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(instance.ty()) else {
            return Err(error("invalid demanded instance"));
        };
        let index = if let Some(template) = templates.get(item) {
            if !instances.insert(instance) {
                continue;
            }
            let input = templates
                .input(item)
                .ok_or_else(|| error("missing checked template input"))?;
            budget.enter(item, &template.body, input).map_err(error)?;
            let mut materialized =
                body::instantiate::with_contract(facts, template, instance, input)
                    .map_err(error)?;
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
            let &index = ordinary.get(&item).ok_or_else(|| {
                error("demanded callable has no concrete body or checked template")
            })?;
            if !visited.insert(index) {
                continue;
            }
            index
        };
        let body = program.body(index).map_err(error)?;
        let source = body.source;
        let mut next = dependencies.collect(facts, body).map_err(error)?;
        next.extend(program.cleanup_dependencies(facts, index).map_err(error)?);
        work.extend(next.into_iter().rev().map(|i| Work::Enter(i, source)));
    }
    Ok(program)
}

fn prepare_ordinary(
    report: &Report,
    facts: &mut Declarations,
    templates: &Checked<'_>,
    inputs: Vec<Contract>,
) -> Result<(proof::Builder, HashMap<ItemId, usize>), Error> {
    let mut program = body::build_concrete(report, facts)?;
    body::promote::analyze(report, facts, &mut program)?;
    body::constants::analyze(report, facts, &program)?;
    let mut inputs: HashMap<_, _> = inputs.into_iter().map(|c| (c.source, c)).collect();
    let (mut program, bodies) =
        proof::Builder::new(program).map_err(|e| diagnostic(report, None, e.into()))?;
    let mut ordinary = HashMap::new();
    for body in bodies {
        let source = body.source;
        let definition = if let Owner::Function { definition, .. } = body.owner {
            Some(definition)
        } else {
            None
        };
        let input = inputs.remove(&source).ok_or_else(|| {
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
            ordinary.insert(ItemId(definition.0), index);
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
