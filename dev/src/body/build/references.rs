//! Reference operations preserve places and never snapshot a borrowed referent.
use super::{Builder, Error, NodeId, Place, Rvalue, Type, TypeId, ValueId};
use crate::body::ir::{BorrowKind, ProjectionKind};

impl Builder<'_> {
    pub(super) fn finish_reborrow(&mut self, node: NodeId) -> Result<(), Error> {
        let place = self.pop_place(node)?;
        if self.scratch.unadjusted.pop() != Some(node) {
            return Err(self.error(node, "mismatched reference adjustment continuation"));
        }
        let Some(mut place) = place.filter(|_| self.current.is_some()) else {
            return Ok(());
        };
        let key = super::NodeKey {
            node,
            ..self.body.source
        };
        let reference = self
            .facts
            .reference_adjustments
            .get(&key)
            .copied()
            .ok_or_else(|| self.error(node, "missing reference conversion selection"))?;
        let adjustment = self
            .fact(node)?
            .adjustment
            .ok_or_else(|| self.error(node, "missing reference conversion target"))?;
        if self.place_type(node, place)? != self.ty(node)? {
            return Err(self.error(
                node,
                "reference conversion source differs from selected place",
            ));
        }
        for _ in 0..reference.conversion.dereferences {
            place = self.dereference_place(node, place)?;
        }
        let kind = if reference.conversion.two_phase {
            BorrowKind::TwoPhase
        } else {
            BorrowKind::Immediate
        };
        let value = self.evaluate(node, reference.borrow, Rvalue::Borrow { place, kind })?;
        let value = if self.facts.pointer_adjustments.contains_key(&key) {
            self.selected_pointer(node, value, adjustment.target)?
        } else if self.facts.unsizing_adjustments.contains_key(&key) {
            self.selected_unsizing(node, value, adjustment.target)?
        } else {
            self.evaluate(node, adjustment.target, Rvalue::Constrain(value))?
        };
        self.save(node, value)
    }
    pub(super) fn selected_pointer(
        &mut self,
        node: NodeId,
        value: ValueId,
        target: TypeId,
    ) -> Result<ValueId, Error> {
        let key = super::NodeKey {
            node,
            ..self.body.source
        };
        let intermediate = *self
            .facts
            .pointer_adjustments
            .get(&key)
            .ok_or_else(|| self.error(node, "missing selected pointer conversion"))?;
        let value = self.evaluate(
            node,
            intermediate,
            Rvalue::Cast {
                kind: crate::sema::model::Cast::Pointer,
                operand: value,
            },
        )?;
        if self.facts.unsizing_adjustments.contains_key(&key) {
            self.selected_unsizing(node, value, target)
        } else if intermediate != target {
            self.evaluate(node, target, Rvalue::Constrain(value))
        } else {
            Ok(value)
        }
    }
    pub(super) fn selected_unsizing(
        &mut self,
        node: NodeId,
        value: ValueId,
        target: TypeId,
    ) -> Result<ValueId, Error> {
        let key = super::NodeKey {
            node,
            ..self.body.source
        };
        let array = *self
            .facts
            .unsizing_adjustments
            .get(&key)
            .ok_or_else(|| self.error(node, "missing selected unsizing array"))?;
        self.evaluate(node, target, Rvalue::Unsize { value, array })
    }
    pub(super) fn place_type(&self, node: NodeId, place: Place) -> Result<TypeId, Error> {
        match place {
            Place::Local(id) => Ok(self.body.locals[id.0 as usize].ty),
            Place::Temporary(id) => Ok(self.body.values[id.0 as usize].ty),
            Place::Projection(id) => Ok(self.body.projections[id.0 as usize].ty),
            Place::Static(id) => self.facts.definitions[id.0 as usize]
                .ok_or_else(|| self.error(node, "missing static place type")),
        }
    }
    pub(super) fn dereference_place(&mut self, node: NodeId, base: Place) -> Result<Place, Error> {
        let id = self.dereference_projection(node, base)?;
        self.statement(node, super::Action::Project(id))?;
        Ok(Place::Projection(id))
    }
    pub(super) fn dereference_projection(
        &mut self,
        node: NodeId,
        base: Place,
    ) -> Result<crate::body::ir::ProjectionId, Error> {
        let ty = self.place_type(node, base)?;
        let Some(Type::Reference {
            region,
            mutable,
            referent,
        }) = self.facts.types.kind(ty)
        else {
            return Err(self.error(node, "selected dereference requires a reference place"));
        };
        self.projection(
            node,
            referent,
            base,
            ProjectionKind::Dereference { region, mutable },
        )
    }
    pub(super) fn automatic_dereferences(
        &mut self,
        node: NodeId,
        mut base: Place,
    ) -> Result<Place, Error> {
        let key = super::NodeKey {
            node,
            ..self.body.source
        };
        let count = self.facts.autoderefs.get(&key).copied().unwrap_or(0);
        for _ in 0..count {
            base = self.dereference_place(node, base)?;
        }
        Ok(base)
    }
    pub(super) fn finish_borrow(&mut self, node: NodeId) -> Result<ValueId, Error> {
        let place = self
            .pop_place(node)?
            .ok_or_else(|| self.error(node, "missing borrowed place"))?;
        let ty = self.ty(node)?;
        let Some(Type::Reference { referent, .. }) = self.facts.types.kind(ty) else {
            return Err(self.error(node, "borrow result is not a reference"));
        };
        if self.place_type(node, place)? != referent {
            return Err(self.error(node, "borrow referent differs from selected place"));
        }
        self.evaluate(
            node,
            ty,
            Rvalue::Borrow {
                place,
                kind: BorrowKind::Immediate,
            },
        )
    }
}
