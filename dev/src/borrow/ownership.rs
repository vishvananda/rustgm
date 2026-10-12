//! Move/initialization proof for owned values. Loans and regions remain separate proofs.
mod paths;
mod state;
pub mod view;
pub use crate::body::scalar_check::Error;
use crate::{
    body::ir::{Action, Body, LocalKind, Place, Program, Rvalue},
    types::{
        ItemId, Kind, Store,
        aggregate::Definitions,
        copy::{Copies, Properties},
    },
};
pub use paths::{Path, PathId, Paths, Root};
use state::State;
use std::{collections::HashMap, hash::BuildHasher};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Use {
    #[default]
    Other,
    Copy,
    Move,
}
pub struct BodyFacts {
    pub paths: Paths,
    /// One selection per value; uses retain Copy/Move independently of memory transfers.
    pub uses: Vec<Use>,
    pub reachable: Vec<bool>,
}
pub struct Checked<'a> {
    program: &'a Program,
    pub bodies: Vec<BodyFacts>,
}
impl Checked<'_> {
    #[must_use]
    pub const fn program(&self) -> &Program {
        self.program
    }
}
/// # Errors
/// Rejects moved/uninitialized reads, invalid projected writes, repeated immutable
/// initialization, moving through indexing, and unsatisfied array-repeat Copy bounds.
/// This is not a loan or region proof and must not authorize reference erasure.
pub fn check<'a, S: BuildHasher>(
    types: &Store,
    definitions: &Definitions,
    derives: &HashMap<ItemId, Properties, S>,
    copies: &mut Copies,
    program: &'a Program,
) -> Result<Checked<'a>, Error> {
    let mut bodies = Vec::with_capacity(program.bodies.len());
    for body in &program.bodies {
        let error = |message| Error {
            node: body.source,
            message,
        };
        let mut uses = vec![Use::Other; body.values.len()];
        for (index, value) in body.values.iter().enumerate() {
            if matches!(value.kind, Rvalue::Read(_)) {
                uses[index] = if copies
                    .demand(types, definitions, derives, value.ty)
                    .map_err(error)?
                    .copy
                {
                    Use::Copy
                } else {
                    Use::Move
                };
            }
            if let Rvalue::Repeat { value: operand } = value.kind {
                let Some(Kind::Array { element, length }) = types.kind(value.ty) else {
                    return Err(error("repeat has no canonical array type"));
                };
                // A constant operand is evaluated independently for each element;
                // an ordinary value is evaluated once, including lengths zero/one.
                if length > 1
                    && !matches!(body.values[operand.0 as usize].kind, Rvalue::Constant(_))
                    && !copies
                        .demand(types, definitions, derives, element)
                        .map_err(error)?
                        .copy
                {
                    return Err(error("repeated array element must implement Copy"));
                }
            }
        }
        let mut facts = BodyFacts {
            paths: Paths::build(body).map_err(error)?,
            uses,
            reachable: Vec::new(),
        };
        state::verify(body, &mut facts)?;
        bodies.push(facts);
    }
    Ok(Checked { program, bodies })
}
impl BodyFacts {
    fn transfer(&self, body: &Body, state: &mut State, action: Action) -> Result<(), &'static str> {
        match action {
            Action::StorageLive(id) | Action::StorageDead(id) => {
                let path = &self.paths.entries[id.0 as usize];
                let initialized = matches!(action, Action::StorageLive(_))
                    && matches!(body.locals[id.0 as usize].kind, LocalKind::Parameter(_));
                state.fill(path, initialized)?;
                state.written(path.start, initialized);
            }
            Action::Evaluate(id) => {
                if self.uses[id.0 as usize] == Use::Move
                    && let Rvalue::Read(place) = body.values[id.0 as usize].kind
                {
                    let info = self.paths.place(place)?;
                    if !info.indexed
                        && let Some(path) = info.path
                    {
                        state.fill(&self.paths.entries[path.0 as usize], false)?;
                    }
                }
                if let Some(path) = self.paths.temporaries.get(&id) {
                    state.fill(&self.paths.entries[path.0 as usize], true)?;
                }
            }
            Action::Store { place, .. } => {
                let info = self.paths.place(place)?;
                if !info.indexed
                    && let Some(path) = info.path
                {
                    let path = &self.paths.entries[path.0 as usize];
                    state.fill(path, true)?;
                    if path.parent.is_none() {
                        state.written(path.start, true);
                    }
                }
            }
            Action::Project(_) => {}
        }
        Ok(())
    }
    fn inspect(&self, body: &Body, state: &mut State, action: Action) -> Result<(), &'static str> {
        match action {
            Action::Evaluate(id) => {
                if let Rvalue::Read(place) = body.values[id.0 as usize].kind {
                    let info = self.paths.place(place)?;
                    if let Some(path) = info.path {
                        if !state.initialized(&self.paths.entries[path.0 as usize])? {
                            return Err("local may be uninitialized or moved at this read");
                        }
                    } else if self.uses[id.0 as usize] == Use::Move {
                        return Err("cannot move a non-Copy value out of static storage");
                    }
                    if info.indexed && self.uses[id.0 as usize] == Use::Move {
                        return Err("cannot move a non-Copy value through array indexing");
                    }
                }
            }
            Action::Project(id) => {
                let info = self.paths.projections[id.0 as usize];
                if info.indexed
                    && let Some(path) = info.path
                    && !state.initialized(&self.paths.entries[path.0 as usize])?
                {
                    return Err("indexed array may be uninitialized or moved");
                }
            }
            Action::Store { place, .. } => self.inspect_store(body, state, place)?,
            _ => {}
        }
        Ok(())
    }
    fn inspect_store(
        &self,
        body: &Body,
        state: &mut State,
        place: Place,
    ) -> Result<(), &'static str> {
        let info = self.paths.place(place)?;
        let Some(path) = info.path else {
            return Err("writing static storage requires unsafe checking");
        };
        let path = &self.paths.entries[path.0 as usize];
        let projected = path.parent.is_some() || info.indexed;
        if let Root::Local(local) = path.root
            && !body.locals[local.0 as usize].mutable
        {
            if projected {
                return Err("cannot assign through an immutable local");
            }
            if state.was_written(path.start) {
                return Err("cannot assign twice to an immutable local");
            }
        }
        if let Some(parent) = path.parent
            && !state.shallow(self.paths.entries[parent.0 as usize].start)
        {
            return Err("cannot assign a field of an uninitialized or wholly moved value");
        }
        if info.indexed && !state.initialized(path)? {
            return Err("indexed array may be uninitialized or moved");
        }
        Ok(())
    }
}
