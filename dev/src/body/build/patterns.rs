//! Pattern bindings read selected places; wildcard fields never load their source.
use super::{Builder, Error, LocalKind, NodeId, Place, Rvalue};
use crate::{
    body::ir::ProjectionKind,
    sema::patterns::{Capture, Kind, PatternId, Projection},
};
pub(super) enum Work {
    Enter(PatternId, Option<Place>, Option<Projection>),
    Capture(PatternId, Option<Place>),
}
impl Builder<'_> {
    pub(super) fn bind_pattern(
        &mut self,
        root: PatternId,
        place: Option<Place>,
    ) -> Result<(), Error> {
        self.scratch
            .pattern_work
            .push(Work::Enter(root, place, None));
        while let Some(work) = self.scratch.pattern_work.pop() {
            match work {
                Work::Enter(id, base, selection) => self.pattern_step(id, base, selection)?,
                Work::Capture(id, place) => self.pattern_capture(id, place)?,
            }
        }
        Ok(())
    }
    fn pattern_step(
        &mut self,
        id: PatternId,
        base: Option<Place>,
        selection: Option<Projection>,
    ) -> Result<(), Error> {
        let pattern = &self.facts.patterns.nodes[id.0 as usize];
        let node = pattern.source.node;
        let ty = pattern
            .ty
            .ok_or_else(|| self.error(node, "unsolved pattern type"))?;
        let place = if let (Some(base), Some(selection)) = (base, selection) {
            let projection = match selection {
                Projection::Field(field) => ProjectionKind::Field(field),
                Projection::Index(index) => ProjectionKind::ConstantIndex(index),
                Projection::Subarray { start, end } => ProjectionKind::Subarray { start, end },
                Projection::Subslice { .. } => {
                    return Err(self.error(
                        node,
                        "slice pattern execution requires region and loan proofs",
                    ));
                }
            };
            Some(self.project(node, ty, base, projection)?)
        } else {
            base
        };
        match pattern.kind {
            Kind::Dereference { .. }
            | Kind::Binding {
                capture: Capture::Borrow { .. },
                ..
            } => {
                return Err(self.error(
                    node,
                    "reference pattern execution requires region and loan proofs",
                ));
            }
            Kind::Wildcard => {}
            Kind::Binding { subpattern, .. } => {
                self.scratch.pattern_work.push(Work::Capture(id, place));
                // Copies from subpatterns happen before the containing move.
                // Two overlapping non-Copy bindings still fail the owned-value proof.
                if let Some(child) = subpattern {
                    self.scratch
                        .pattern_work
                        .push(Work::Enter(child, place, None));
                }
            }
            Kind::Product { fields, .. } => {
                for field in self.facts.patterns.fields(fields).iter().rev() {
                    self.scratch.pattern_work.push(Work::Enter(
                        field.pattern,
                        place,
                        Some(field.selection),
                    ));
                }
            }
            Kind::Or(alternatives) => {
                // This irrefutable-product profile has no refutable alternatives.
                // Conditional patterns will need a decision CFG before binding.
                let first = self
                    .facts
                    .patterns
                    .alternatives(alternatives)
                    .first()
                    .copied()
                    .ok_or_else(|| self.error(node, "empty typed alternative pattern"))?;
                self.scratch
                    .pattern_work
                    .push(Work::Enter(first, place, None));
            }
            Kind::Pending => return Err(self.error(node, "incomplete pattern selection")),
        }
        Ok(())
    }
    fn pattern_capture(&mut self, id: PatternId, place: Option<Place>) -> Result<(), Error> {
        let pattern = &self.facts.patterns.nodes[id.0 as usize];
        let node = pattern.source.node;
        let Kind::Binding {
            definition,
            mutable,
            ..
        } = pattern.kind
        else {
            return Err(self.error(node, "missing selected pattern binding"));
        };
        let ty = pattern
            .ty
            .ok_or_else(|| self.error(node, "unsolved binding type"))?;
        let local = self.local(node, ty, LocalKind::Binding, Some(definition), mutable)?;
        if let Some(place) = place {
            let value = self.evaluate(node, ty, Rvalue::Read(place))?;
            self.store(node, Place::Local(local), value)?;
        }
        Ok(())
    }
    pub(super) fn pattern_root(&self, node: NodeId) -> Option<PatternId> {
        self.facts
            .patterns
            .roots
            .get(&super::NodeKey {
                file: self.body.source.file,
                node,
            })
            .copied()
    }
}
