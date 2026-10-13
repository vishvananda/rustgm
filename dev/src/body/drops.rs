//! Pending destruction requirements; initialization and cleanup order are not proved here.
pub(crate) mod view;
use super::{
    ir::{Action, Body, LocalId, LocalKind, NodeKey, Place, Program},
    storage::Location,
};
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    types::{ItemId, Kind, TypeId, destruction::NeedsDrop},
};

pub struct Method {
    pub item: ItemId,
    pub function: DefId,
    pub target: TypeId,
    pub signature: TypeId,
    pub captures: u32,
    pub source: NodeKey,
}
#[derive(Clone, Copy, Debug)]
pub enum Trigger {
    /// The persistent live-stack range, without copying its member locals.
    Locals { first: LocalId, last: LocalId },
    /// A potential replacement: later initialization analysis selects old-value drops.
    Store(Place),
    /// Index into the body's retained scope/temporary exits.
    Exit(u32),
}
pub struct Site {
    pub location: Location,
    pub trigger: Trigger,
}
pub struct Requirements {
    pub locals: Vec<bool>,
    pub values: Vec<bool>,
    pub projections: Vec<bool>,
    pub sites: Vec<Site>,
}
/// Retaining this value forbids completed borrow proofs and executable lowering.
/// These candidates still need move/init flags, destructor liveness and elaboration.
pub struct Pending {
    pub source: NodeKey,
    pub methods: Vec<Method>,
    pub bodies: Vec<Requirements>,
    pub query_steps: usize,
}
pub const PENDING_MESSAGE: &str = "Drop implementations require destructor elaboration";

pub(crate) fn collect(
    facts: &Declarations,
    program: &Program,
) -> Result<Option<Pending>, (NodeKey, &'static str)> {
    let Some(first) = facts.destructors.implementations().first() else {
        return Ok(None);
    };
    let mut methods = Vec::new();
    for implementation in facts.destructors.implementations() {
        let Some(Kind::FunctionItem { signature, .. }) = facts
            .definitions
            .get(implementation.method.0 as usize)
            .copied()
            .flatten()
            .and_then(|ty| facts.types.kind(ty))
        else {
            return Err((implementation.source, "missing destructor signature"));
        };
        methods.push(Method {
            item: implementation.item,
            function: implementation.method,
            target: implementation.target,
            signature,
            captures: implementation.parameters,
            source: implementation.source,
        });
    }
    let mut query = NeedsDrop::new(methods.iter().map(|method| method.item));
    let bodies = program
        .bodies
        .iter()
        .map(|body| requirements(facts, body, &mut query).map_err(|message| (body.source, message)))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(Pending {
        source: first.source,
        methods,
        bodies,
        query_steps: query.steps,
    }))
}
fn requirements(
    facts: &Declarations,
    body: &Body,
    query: &mut NeedsDrop,
) -> Result<Requirements, &'static str> {
    let mut needs = |ty| query.demand(&facts.types, &facts.aggregates, ty);
    let locals = body
        .locals
        .iter()
        .map(|local| {
            if matches!(local.kind, LocalKind::Guard(_)) {
                Ok(false)
            } else {
                needs(local.ty)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    let values = body
        .values
        .iter()
        .map(|value| needs(value.ty))
        .collect::<Result<Vec<_>, _>>()?;
    let projections = body
        .projections
        .iter()
        .map(|p| needs(p.ty))
        .collect::<Result<Vec<_>, _>>()?;
    let mut result = Requirements {
        locals,
        values,
        projections,
        sites: Vec::new(),
    };
    if !result.locals.contains(&true)
        && !result.values.contains(&true)
        && !result.projections.contains(&true)
    {
        return Ok(result);
    }
    let mut counts: Vec<u32> = Vec::with_capacity(body.locals.len());
    for (index, local) in body.locals.iter().enumerate() {
        let parent = local.storage_parent.map_or(Ok(0), |id| {
            counts
                .get(id.0 as usize)
                .copied()
                .ok_or("invalid destruction storage ancestry")
        })?;
        counts.push(
            parent
                .checked_add(u32::from(result.locals[index]))
                .ok_or("destruction storage count overflow")?,
        );
    }
    for (index, block) in body.blocks.iter().enumerate() {
        for before in block.start..block.end {
            let statement = body
                .statements
                .get(before as usize)
                .ok_or("invalid destruction statement range")?;
            let trigger = match statement.action {
                Action::StorageDead(local)
                    if result
                        .locals
                        .get(local.0 as usize)
                        .copied()
                        .ok_or("invalid destruction local")? =>
                {
                    Some(Trigger::Locals {
                        first: local,
                        last: local,
                    })
                }
                Action::StorageEnd { first, last }
                    if nonempty_range(body, &counts, first, last)? =>
                {
                    Some(Trigger::Locals { first, last })
                }
                Action::Store { place, .. } if place_needs(facts, &result, query, place)? => {
                    Some(Trigger::Store(place))
                }
                _ => None,
            };
            if let Some(trigger) = trigger {
                let block = super::ir::BlockId(
                    u32::try_from(index).map_err(|_| "too many destruction blocks")?,
                );
                result.sites.push(Site {
                    location: Location::At { block, before },
                    trigger,
                });
            }
        }
    }
    if let Some(storage) = &body.storage {
        for (index, exit) in storage.exits.iter().enumerate() {
            result.sites.push(Site {
                location: exit.location,
                trigger: Trigger::Exit(
                    u32::try_from(index).map_err(|_| "too many destruction exits")?,
                ),
            });
        }
    }
    Ok(result)
}
fn nonempty_range(
    body: &Body,
    counts: &[u32],
    first: LocalId,
    last: LocalId,
) -> Result<bool, &'static str> {
    let first = body
        .locals
        .get(first.0 as usize)
        .ok_or("invalid destruction storage range")?;
    let before = first.storage_parent.map_or(Ok(0), |id| {
        counts
            .get(id.0 as usize)
            .copied()
            .ok_or("invalid destruction storage predecessor")
    })?;
    let after = counts
        .get(last.0 as usize)
        .copied()
        .ok_or("invalid destruction storage endpoint")?;
    Ok(after > before)
}
fn place_needs(
    facts: &Declarations,
    body: &Requirements,
    query: &mut NeedsDrop,
    place: Place,
) -> Result<bool, &'static str> {
    let value = match place {
        Place::Local(id) => body.locals.get(id.0 as usize),
        Place::Temporary(id) => body.values.get(id.0 as usize),
        Place::Projection(id) => body.projections.get(id.0 as usize),
        Place::Static(id) => {
            let ty = facts
                .definitions
                .get(id.0 as usize)
                .copied()
                .flatten()
                .ok_or("missing static destruction type")?;
            return query.demand(&facts.types, &facts.aggregates, ty);
        }
    };
    value.copied().ok_or("invalid destruction place")
}
