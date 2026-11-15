//! A destructuring assignment evaluates its right operand into a place, reads
//! the projected element of every place leaf, then stores each to its leaf from
//! left to right, as rustc's `{ let <pattern> = rhs; leaf = binding; ... }`
//! does. Reading first keeps a store from changing an element that a later
//! leaf takes when a leaf overlaps the right operand's place. A leaf place is
//! evaluated after the stores before it; `_` and unread elements stay in the
//! right operand's place.
use super::{Builder, Error, NodeId, Place, Rvalue, Type, ValueId, Work};
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
        let mut stores: Vec<(NodeId, Option<ValueId>)> = Vec::new();
        for entry in &walk {
            let place = match entry.parent {
                None => source,
                Some((parent, element)) => {
                    let (shell, base) = walk
                        .get(parent as usize)
                        .zip(places.get(parent as usize))
                        .ok_or_else(|| self.error(node, "invalid assignee walk"))?;
                    match *base {
                        Some(base) if self.current.is_some() && entry.role != Role::Wildcard => {
                            Some(self.shell_element(node, shell.role, base, element)?)
                        }
                        _ => None,
                    }
                }
            };
            places.push(place);
            match entry.role {
                Role::Place => {
                    let value = match place {
                        Some(place) if self.current.is_some() => {
                            let ty = self.place_type(node, place)?;
                            Some(self.evaluate(node, ty, Rvalue::Read(place))?)
                        }
                        _ => None,
                    };
                    stores.push((entry.node, value));
                }
                Role::Rest => {
                    return Err(self.error(entry.node, "rest assignees are not implemented"));
                }
                Role::Tuple | Role::Array | Role::Wildcard => {}
            }
        }
        // The worklist is LIFO, so the first leaf's place and store run first.
        for (leaf, value) in stores.into_iter().rev() {
            self.scratch
                .work
                .push(Work::DestructureStore { node, leaf, value });
            self.scratch.work.push(Work::Place(leaf));
        }
        Ok(())
    }
    /// A tuple shell selects a field; an array shell selects a constant index,
    /// as an array pattern's element does.
    fn shell_element(
        &mut self,
        node: NodeId,
        shell: Role,
        base: Place,
        element: u32,
    ) -> Result<Place, Error> {
        let aggregate = self.place_type(node, base)?;
        let selected = match (shell, self.facts.types.kind(aggregate)) {
            (Role::Tuple, Some(Type::Tuple(fields))) => self
                .facts
                .types
                .list(fields)
                .and_then(|fields| fields.get(element as usize))
                .map(|&ty| (ty, ProjectionKind::Field(Selection::Tuple(element)))),
            (
                Role::Array,
                Some(Type::Array {
                    element: ty,
                    length,
                }),
            ) if u64::from(element) < length => {
                Some((ty, ProjectionKind::ConstantIndex(u64::from(element))))
            }
            _ => None,
        };
        let (ty, kind) = selected.ok_or_else(|| {
            self.error(
                node,
                "destructured value does not have the assignee's shape",
            )
        })?;
        self.project(node, ty, base, kind)
    }
    /// Store one element, read before any leaf's store, into its leaf.
    pub(super) fn destructure_store(
        &mut self,
        node: NodeId,
        leaf: NodeId,
        value: Option<ValueId>,
    ) -> Result<(), Error> {
        let target = self.pop_place(leaf)?;
        let (Some(target), Some(value)) = (target, value) else {
            return Ok(());
        };
        if self.current.is_none() {
            return Ok(());
        }
        let ty = self.body.values[value.0 as usize].ty;
        let target_ty = self.place_type(leaf, target)?;
        let value = if ty == target_ty {
            value
        } else {
            self.evaluate(node, target_ty, Rvalue::Constrain(value))?
        };
        self.store(node, target, value)
    }
}
