//! Move/initialization proof for owned values. Loans and regions remain separate proofs.
mod guards;
mod paths;
mod state;
pub mod view;
pub struct Error {
    pub node: crate::body::ir::NodeKey,
    pub message: &'static str,
}
use crate::{
    body::ir::{Action, Body, LocalKind, Place, Program, ProjectionKind, Rvalue},
    types::{
        ItemId, Kind, Store,
        aggregate::Definitions,
        copy::{Copies, Properties},
    },
};
use paths::Access;
pub use paths::{Component, Path, PathId, Paths, Root};
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
}
pub struct Analysis<'a> {
    checked: Checked<'a>,
    pub bodies: Vec<BodyFacts>,
}
impl Analysis<'_> {
    #[must_use]
    pub const fn program(&self) -> &Program {
        self.checked.program
    }
    #[must_use]
    pub const fn checked(&self) -> &Checked<'_> {
        &self.checked
    }
}
impl Checked<'_> {
    #[must_use]
    pub const fn program(&self) -> &Program {
        self.program
    }
}
impl<'a> Analysis<'a> {
    pub(super) fn from_proved(program: &'a Program, bodies: Vec<BodyFacts>) -> Self {
        Self {
            checked: Checked { program },
            bodies,
        }
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
) -> Result<Analysis<'a>, Error> {
    let mut bodies = Vec::with_capacity(program.bodies.len());
    let checked = visit(types, definitions, derives, copies, program, |facts| {
        bodies.push(facts);
    })?;
    Ok(Analysis { checked, bodies })
}
/// Run the identical proof while releasing each body's inspection records before
/// executable lowering. Later analyses that need move paths use `check` instead.
///
/// # Errors
/// Reports the same ownership, initialization and resource errors as `check`.
pub fn verify<'a, S: BuildHasher>(
    types: &Store,
    definitions: &Definitions,
    derives: &HashMap<ItemId, Properties, S>,
    copies: &mut Copies,
    program: &'a Program,
) -> Result<Checked<'a>, Error> {
    visit(types, definitions, derives, copies, program, drop)
}
fn visit<'a, S: BuildHasher>(
    types: &Store,
    definitions: &Definitions,
    derives: &HashMap<ItemId, Properties, S>,
    copies: &mut Copies,
    program: &'a Program,
    mut record: impl FnMut(BodyFacts),
) -> Result<Checked<'a>, Error> {
    let mut pending = None;
    for body in &program.bodies {
        let facts = analyze_body(types, definitions, derives, copies, body)?;
        record(facts);
        if pending.is_none()
            && (body
                .values
                .iter()
                .any(|v| matches!(v.kind, Rvalue::Borrow { .. }))
                || body
                    .projections
                    .iter()
                    .any(|p| matches!(p.kind, ProjectionKind::Dereference { .. })))
        {
            pending = Some(body.source);
        }
    }
    if let Some(node) = pending {
        return Err(Error {
            node,
            message: "reference expression execution requires region and loan proofs",
        });
    }
    Ok(Checked { program })
}
/// Prove owned initialization/access facts for one body without granting an
/// executable token. Loan and lifetime checks consume these same typed facts.
///
/// # Errors
/// Rejects invalid moves, initialization, permissions and resource exhaustion.
pub fn analyze_body<S: BuildHasher>(
    types: &Store,
    definitions: &Definitions,
    derives: &HashMap<ItemId, Properties, S>,
    copies: &mut Copies,
    body: &Body,
) -> Result<BodyFacts, Error> {
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
        paths: Paths::build(body, types).map_err(error)?,
        uses,
        reachable: Vec::new(),
    };
    state::verify(body, &mut facts, types)?;
    guards::verify(body, &facts, types)?;
    Ok(facts)
}
impl BodyFacts {
    fn transfer(&self, body: &Body, state: &mut State, action: Action) -> Result<(), &'static str> {
        match action {
            Action::BindView { view, .. } => {
                let path = &self.paths.entries[view.0 as usize];
                state.fill(path, true)?;
                state.written(path.start, true);
            }
            Action::StorageLive(id) | Action::StorageDead(id) => {
                let path = &self.paths.entries[id.0 as usize];
                let initialized = matches!(action, Action::StorageLive(_))
                    && matches!(body.locals[id.0 as usize].kind, LocalKind::Parameter(_));
                state.fill(path, initialized)?;
                state.written(path.start, initialized);
            }
            Action::StorageEnd { first, last } => {
                for local in body.storage_range(first, last) {
                    let path = &self.paths.entries[local?.0 as usize];
                    state.fill(path, false)?;
                    state.written(path.start, false);
                }
            }
            Action::Evaluate(id) => {
                if self.uses[id.0 as usize] == Use::Move
                    && let Rvalue::Read(place) = body.values[id.0 as usize].kind
                {
                    let info = self.paths.place(place)?;
                    if !info.indexed
                        && info.access == Access::Owned
                        && !info.empty
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
                    && info.access == Access::Owned
                    && let Some(path) = info.path
                {
                    let path = &self.paths.entries[path.0 as usize];
                    state.fill(path, true)?;
                    if path.parent.is_none() {
                        state.written(path.start, true);
                    }
                }
            }
            Action::GuardStart(_)
            | Action::GuardUse(_)
            | Action::Project(_)
            | Action::Select { .. } => {}
        }
        Ok(())
    }
    fn inspect(
        &self,
        body: &Body,
        types: &Store,
        state: &mut State,
        action: Action,
    ) -> Result<(), &'static str> {
        match action {
            Action::GuardUse(id) => {
                let guard = body
                    .guards
                    .get(id.0 as usize)
                    .ok_or("missing guard use owner")?;
                for local in guard.views_start..guard.views_end {
                    let path = self
                        .paths
                        .entries
                        .get(local as usize)
                        .ok_or("invalid guard view")?;
                    if !state.initialized(path)? {
                        return Err("guard binding may be uninitialized at guard completion");
                    }
                }
            }
            Action::BindView { view, place } => {
                if !matches!(
                    body.locals.get(view.0 as usize).map(|l| l.kind),
                    Some(LocalKind::Guard(_))
                ) {
                    return Err("view binding requires an internal guard local");
                }
                let (path, shallow) = self.paths.guard_target(self.paths.place(place)?);
                if let Some(path) = path {
                    let path = &self.paths.entries[path.0 as usize];
                    let initialized = if shallow {
                        state.shallow(path.start)
                    } else {
                        state.initialized(path)?
                    };
                    if !initialized {
                        return Err("guard binding may be uninitialized or moved");
                    }
                }
            }
            Action::Evaluate(id) => self.inspect_value(body, types, state, id)?,
            Action::Project(id) | Action::Select { projection: id, .. } => {
                let info = self.paths.projections[id.0 as usize];
                if (info.indexed || info.access != Access::Owned)
                    && let Some(path) = info.path
                    && !state.initialized(&self.paths.entries[path.0 as usize])?
                {
                    return Err(if info.access == Access::Owned {
                        "indexed array may be uninitialized or moved"
                    } else {
                        "dereferenced pointer may be uninitialized or moved"
                    });
                }
            }
            Action::Store { place, .. } => self.inspect_store(body, state, place)?,
            _ => {}
        }
        Ok(())
    }
    fn inspect_value(
        &self,
        body: &Body,
        types: &Store,
        state: &mut State,
        id: crate::body::ir::ValueId,
    ) -> Result<(), &'static str> {
        if let Rvalue::Borrow { place, .. } = body.values[id.0 as usize].kind {
            let Some(Kind::Reference { mutable, .. }) = types.kind(body.values[id.0 as usize].ty)
            else {
                return Err("borrow value has no reference type");
            };
            let info = self.paths.place(place)?;
            let (path, shallow) = self.paths.guard_target(info);
            if let Some(path) = path {
                let path = &self.paths.entries[path.0 as usize];
                let initialized = if shallow {
                    state.shallow(path.start)
                } else {
                    state.initialized(path)?
                };
                if !initialized {
                    return Err("borrowed place may be uninitialized or moved");
                }
            }
            if mutable {
                self.mutable_borrow(body, info)?;
            }
        }
        if let Rvalue::IsVariant { place, .. }
        | Rvalue::Length(place)
        | Rvalue::TextEquals { place, .. }
        | Rvalue::Validate(place) = body.values[id.0 as usize].kind
        {
            let info = self.paths.place(place)?;
            if let Some(path) = info.path
                && !state.initialized(&self.paths.entries[path.0 as usize])?
            {
                return Err("pattern input may be uninitialized or moved");
            }
        }
        if let Rvalue::Read(place) = body.values[id.0 as usize].kind {
            let info = self.paths.place(place)?;
            if (!info.empty || info.access != Access::Owned)
                && let Some(path) = info.path
                && !state.initialized(&self.paths.entries[path.0 as usize])?
            {
                return Err(if self.uses[id.0 as usize] == Use::Copy {
                    "local may be uninitialized at this read"
                } else {
                    "local may be uninitialized or moved at this read"
                });
            }
            if self.uses[id.0 as usize] == Use::Move
                && self.paths.view_root(body, info.path).is_some()
            {
                return Err("cannot move a non-Copy value out of a guard binding");
            }
            // Eligibility to move through a place still applies to an
            // empty subarray, even though it reads no initialization bits.
            if info.access != Access::Owned && self.uses[id.0 as usize] == Use::Move {
                return Err("cannot move a non-Copy value out of a reference");
            }
            if info.path.is_none() && self.uses[id.0 as usize] == Use::Move {
                return Err("cannot move a non-Copy value out of static storage");
            }
            if info.indexed && self.uses[id.0 as usize] == Use::Move {
                return Err("cannot move a non-Copy value through array indexing");
            }
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
        if self.paths.view_root(body, info.path).is_some() {
            return Err("cannot assign through a guard binding");
        }
        if info.access != Access::Owned {
            if info.access == Access::Shared {
                return Err("cannot assign through a shared reference");
            }
            return Ok(());
        }
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
    fn mutable_borrow(&self, body: &Body, info: paths::PlaceInfo) -> Result<(), &'static str> {
        if self.paths.view_root(body, info.path).is_some() {
            return Err("cannot mutably borrow through a guard binding");
        }
        if info.access == Access::Shared || info.path.is_none() {
            return Err("cannot mutably borrow shared or static storage");
        }
        if info.access == Access::Owned
            && let Some(path) = info.path
            && let Root::Local(local) = self.paths.entries[path.0 as usize].root
        {
            let local = &body.locals[local.0 as usize];
            // An unnamed parameter is owned input storage used by a pattern,
            // just like an rvalue temporary, rather than an immutable binding.
            let input = local.definition.is_none() && matches!(local.kind, LocalKind::Parameter(_));
            if !local.mutable && !input {
                return Err("cannot mutably borrow an immutable local");
            }
        }
        Ok(())
    }
}
