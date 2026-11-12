//! A destructuring assignment evaluates its right operand into a place, then
//! stores each projected field to its place leaf from left to right, as
//! rustc's `{ let <pattern> = rhs; leaf = binding; ... }` does. A leaf place is
//! evaluated after the stores before it; `_` and unread fields stay in the
//! right operand's place.
use super::{Builder, Error, NodeId, Place, Rvalue, Type, TypeId, Work};
use crate::{
    body::ir::ProjectionKind,
    sema::members::Selection,
    syntax::{
        assignee::{Assignee, Role},
        ast::Kind,
    },
};

impl Builder<'_> {
    pub(super) fn destructure(&mut self, node: NodeId) -> Result<(), Error> {
        let source = self.pop_place(node)?;
        let Kind::Binary { left, .. } = self.kind(node) else {
            return Err(self.error(node, "invalid destructuring assignment"));
        };
        let mut walk: Vec<Assignee> = Vec::new();
        self.unit
            .configured
            .ast
            .assignees(left, &self.unit.configured.active, &mut walk);
        let mut places: Vec<Option<Place>> = Vec::with_capacity(walk.len());
        for entry in &walk {
            let place = match entry.parent {
                None => source,
                Some((parent, field)) => {
                    let base = places
                        .get(parent as usize)
                        .copied()
                        .ok_or_else(|| self.error(node, "invalid assignee walk"))?;
                    match base {
                        Some(base) if self.current.is_some() && entry.role != Role::Wildcard => {
                            Some(self.tuple_field(node, base, field)?)
                        }
                        _ => None,
                    }
                }
            };
            places.push(place);
        }
        // The worklist is LIFO, so the first leaf's place and store run first.
        for (entry, source) in walk.iter().zip(places).rev() {
            match entry.role {
                Role::Place => {
                    self.scratch.work.push(Work::DestructureStore {
                        node,
                        leaf: entry.node,
                        source,
                    });
                    self.scratch.work.push(Work::Place(entry.node));
                }
                Role::Rest => {
                    return Err(self.error(entry.node, "rest assignees are not implemented"));
                }
                Role::Tuple | Role::Wildcard => {}
            }
        }
        Ok(())
    }
    fn tuple_field(&mut self, node: NodeId, base: Place, field: u32) -> Result<Place, Error> {
        let ty = self.place_type(node, base)?;
        let element = match self.facts.types.kind(ty) {
            Some(Type::Tuple(fields)) => self
                .facts
                .types
                .list(fields)
                .and_then(|fields| fields.get(field as usize))
                .copied(),
            _ => None,
        }
        .ok_or_else(|| {
            self.error(
                node,
                "destructured value is not a tuple of the assignee's arity",
            )
        })?;
        self.project(
            node,
            element,
            base,
            ProjectionKind::Field(Selection::Tuple(field)),
        )
    }
    /// Move or copy one projected field into its leaf.
    pub(super) fn destructure_store(
        &mut self,
        node: NodeId,
        leaf: NodeId,
        source: Option<Place>,
    ) -> Result<(), Error> {
        let target = self.pop_place(leaf)?;
        let (Some(target), Some(source)) = (target, source) else {
            return Ok(());
        };
        if self.current.is_none() {
            return Ok(());
        }
        let ty: TypeId = self.place_type(node, source)?;
        let value = self.evaluate(node, ty, Rvalue::Read(source))?;
        let target_ty = self.place_type(leaf, target)?;
        let value = if ty == target_ty {
            value
        } else {
            self.evaluate(node, target_ty, Rvalue::Constrain(value))?
        };
        self.store(node, target, value)
    }
}
