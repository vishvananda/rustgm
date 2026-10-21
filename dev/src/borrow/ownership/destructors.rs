//! A custom destructor keeps the whole nominal object intact until destruction.
use super::{BodyFacts, Error, Use};
use crate::{
    body::ir::{Action, Body, Rvalue},
    types::{ItemId, Kind, Store},
};
pub(super) fn check(
    types: &Store,
    custom: impl Fn(ItemId) -> bool,
    body: &Body,
    facts: &BodyFacts,
) -> Result<(), Error> {
    let error = |message| Error {
        node: body.source,
        message,
    };
    let mut protected = Vec::with_capacity(facts.paths.entries.len());
    for path in &facts.paths.entries {
        let guard = if let Some(parent) = path.parent {
            let inherited = protected
                .get(parent.0 as usize)
                .copied()
                .ok_or_else(|| error("destruction path precedes its parent"))?;
            let ty = facts.paths.entries[parent.0 as usize].ty;
            inherited || matches!(types.kind(ty), Some(Kind::Adt(item, _)) if custom(item))
        } else {
            false
        };
        protected.push(guard);
    }
    for statement in &body.statements {
        let Action::Evaluate(value) = statement.action else {
            continue;
        };
        if facts.uses[value.0 as usize] != Use::Move {
            continue;
        }
        let Rvalue::Read(place) = body.values[value.0 as usize].kind else {
            continue;
        };
        let info = facts.paths.place(place).map_err(error)?;
        if info.path.is_some_and(|path| {
            protected[path.0 as usize]
                || info.union && matches!(types.kind(facts.paths.entries[path.0 as usize].ty), Some(Kind::Adt(item, _)) if custom(item))
        }) {
            let mut node = body.source;
            node.node = statement.source;
            return Err(Error {
                node,
                message: "cannot move a non-Copy field out of a type that implements Drop",
            });
        }
    }
    Ok(())
}
