//! Prove actual parents before demanding independent conditional child values.
use super::{Ordinary, diagnostic, proof};
use crate::support::hash::{HashMap, HashSet};
use crate::{
    body::{
        conditional_inputs::Pending,
        ir::{BodyKey, Owner, Program},
        regions::Contract,
    },
    borrow::templates::Checked,
    pipeline::execution::Context,
    resolve::{expansion::Report, model::DefId, package::Error},
    sema::model::Declarations,
    types::ItemId,
};
use std::sync::Arc;

pub(super) fn prepare(
    facts: &mut Declarations,
    templates: &Checked<'_>,
    program: &Program,
    inputs: &mut HashMap<BodyKey, Contract>,
) -> Result<HashMap<BodyKey, Pending>, &'static str> {
    let mut sources = crate::body::conditional_inputs::Sources::default();
    let mut pending = HashMap::default();
    let result = (|| {
        templates.conditional_sources(&mut sources)?;
        if sources.is_empty() {
            return Ok(pending);
        }
        for body in &program.bodies {
            let input = inputs
                .get_mut(&body.key())
                .ok_or("conditional parent has no body contract")?;
            if let Some(ports) = sources.prepare(facts, body, input)? {
                pending.insert(body.key(), ports);
            }
        }
        Ok(pending)
    })();
    facts.body_work.canonical_steps = facts
        .body_work
        .canonical_steps
        .saturating_add(sources.steps);
    result
}

#[derive(Clone, Copy)]
pub(super) struct Demands<'a> {
    pub context: &'a Context,
    pub globals: &'a [DefId],
}
pub(super) fn build(
    report: &Report,
    facts: &mut Declarations,
    templates: &Checked<'_>,
    mut program: Program,
    inputs: HashMap<BodyKey, Contract>,
    mut pending: HashMap<BodyKey, Pending>,
    demand: Demands<'_>,
) -> Result<(proof::Builder, Ordinary), Error> {
    let Demands { context, globals } = demand;
    let error = |message: &str| diagnostic(report, None, message.into());
    let bytes = staging_bytes(&program, &inputs, &pending).map_err(error)?;
    let skip: HashSet<_> = pending.keys().copied().collect();
    context.constants_except(report, facts, &mut program, globals, &skip)?;
    let inputs: HashMap<_, _> = inputs
        .into_iter()
        .map(|(key, input)| (key, Arc::new(input)))
        .collect();
    let (mut builder, bodies) = proof::Builder::new(program).map_err(error)?;
    builder
        .prepare_parent_regions(facts, &inputs, &bodies)
        .map_err(|e| diagnostic(report, e.node, e.message))?;
    let mut staged = Vec::with_capacity(bodies.len());
    for body in bodies {
        let ports = pending.remove(&body.key());
        staged.push(
            builder
                .stage_parent(facts, templates, body, ports)
                .map_err(|e| diagnostic(report, e.node, e.message))?,
        );
    }
    if !pending.is_empty() {
        return Err(error("conditional parent staging lost an owner"));
    }
    // The borrowed group retains every actual ancestor/sibling Proved throughout
    // value demand. No partial stage can produce ParentMember activation.
    let mut work = 0;
    let result: Result<(), Error> = (|| {
        let group = crate::borrow::incremental::CompleteGroup::new(
            staged.iter().map(proof::Staged::paired),
            (64 * 1024 * 1024_usize).saturating_sub(bytes),
            &mut work,
        )
        .map_err(error)?;
        for row in &staged {
            if let proof::Staged::Parent(parent) = row {
                let member = group.parent(parent).map_err(error)?;
                context.with_parent(report, facts, &member)?;
            }
        }
        Ok(())
    })();
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work);
    result?;
    let mut ordinary = Ordinary::default();
    for staged in staged {
        let owner = staged.body().owner;
        let proved = match staged {
            proof::Staged::Ordinary(proved) => (*proved).into_proved(),
            proof::Staged::Parent(parent) => parent.into_proved().map_err(|(_, e)| error(e))?,
        };
        let index = builder
            .append_proved(facts, proved)
            .map_err(|e| diagnostic(report, e.node, e.message))?;
        match owner {
            Owner::Function { definition, .. } => {
                ordinary.functions.insert(ItemId(definition.0), index);
            }
            Owner::Closure { identity, .. } => {
                ordinary.closures.insert(identity, index);
            }
            _ => {}
        }
    }
    Ok((builder, ordinary))
}

fn staging_bytes(
    program: &Program,
    inputs: &HashMap<BodyKey, Contract>,
    pending: &HashMap<BodyKey, Pending>,
) -> Result<usize, &'static str> {
    let bytes = program
        .bodies
        .len()
        .saturating_mul(
            size_of::<proof::Staged>() + size_of::<crate::borrow::incremental::PairedProved>(),
        )
        .saturating_add(
            pending
                .values()
                .map(Pending::retained_bytes)
                .fold(0_usize, usize::saturating_add),
        );
    let bytes = bytes
        .saturating_add(
            pending
                .len()
                .saturating_mul(crate::borrow::incremental::OwnedParent::storage_overhead()),
        )
        .saturating_add(
            inputs
                .values()
                .map(Contract::retained_bytes)
                .fold(0_usize, usize::saturating_add),
        )
        .saturating_add(crate::support::retained::new_map::<BodyKey, Pending>(
            pending.len(),
        ))
        .saturating_add(
            inputs
                .values()
                .map(|input| {
                    crate::support::retained::new_map::<
                        (crate::body::ir::NodeKey, crate::types::outlives::Pair),
                        (),
                    >(input.obligations.len())
                })
                .fold(0_usize, usize::saturating_add),
        );
    if bytes > 64 * 1024 * 1024 || program.bodies.len() > 1_000_000 {
        return Err("conditional parent staging exceeds storage budget");
    }
    Ok(bytes)
}
