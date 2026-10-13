//! Pattern bindings read selected places; wildcard fields never load their source.
use super::{Builder, Error, LocalKind, NodeId, Place, Rvalue, TypeId, ValueId};
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
        if let Some(place) = place
            && self.binding_requires_decision(root)
        {
            let node = self.facts.patterns.nodes[root.0 as usize].source.node;
            let yes = self.new_block(node)?;
            let no = self.new_block(node)?;
            let plan = self.constructor_decision(
                root,
                place,
                yes,
                no,
                super::decisions::Failure::Runtime,
            )?;
            self.switch(node, no)?;
            self.terminate(node, super::Terminator::Unreachable);
            self.switch(node, yes)?;
            return self.constructor_captures(&plan, root, place);
        }
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
    fn binding_requires_decision(&mut self, root: PatternId) -> bool {
        let mut alternatives = false;
        let mut borrows = false;
        self.scratch.match_patterns.clear();
        self.scratch.match_patterns.push(root);
        while let Some(id) = self.scratch.match_patterns.pop() {
            match self.facts.patterns.nodes[id.0 as usize].kind {
                Kind::Range(_) => {
                    self.scratch.match_patterns.clear();
                    return true;
                }
                Kind::Product { variant, fields } => {
                    if let Some(variant) = variant
                        && self
                            .facts
                            .members
                            .variants(self.facts.members.variants[variant.0 as usize].owner)
                            .len()
                            > 1
                    {
                        self.scratch.match_patterns.clear();
                        return true;
                    }
                    self.scratch
                        .match_patterns
                        .extend(self.facts.patterns.fields(fields).iter().map(|f| f.pattern));
                }
                Kind::Or(choices) => {
                    alternatives = true;
                    self.scratch
                        .match_patterns
                        .extend(self.facts.patterns.alternatives(choices));
                }
                Kind::Binding {
                    capture,
                    subpattern,
                    ..
                } => {
                    borrows |= matches!(capture, Capture::Borrow { .. });
                    self.scratch.match_patterns.extend(subpattern);
                }
                Kind::Dereference { pattern, .. } => self.scratch.match_patterns.push(pattern),
                _ => {}
            }
        }
        // Both occurrences must retain provenance even if the first alternative
        // is irrefutable and always selected at runtime.
        alternatives && borrows
    }
    fn pattern_step(
        &mut self,
        id: PatternId,
        base: Option<Place>,
        selection: Option<Projection>,
    ) -> Result<(), Error> {
        let pattern = &self.facts.patterns.nodes[id.0 as usize];
        let node = pattern.source.node;
        let kind = pattern.kind;
        let place = if let Some(base) = base {
            Some(self.selected_place(id, base, selection)?)
        } else {
            base
        };
        match kind {
            Kind::Range(_) if place.is_some() => {
                return Err(self.error(
                    node,
                    "conditional binding execution requires pattern decisions",
                ));
            }
            Kind::Text(_) | Kind::Slice { .. } => {
                return Err(self.error(
                    node,
                    "reference pattern execution requires region and loan proofs",
                ));
            }
            Kind::Wildcard | Kind::Range(_) => {}
            Kind::Dereference { pattern, .. } => {
                let place = place
                    .map(|base| self.dereference_place(node, base))
                    .transpose()?;
                self.scratch
                    .pattern_work
                    .push(Work::Enter(pattern, place, None));
            }
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
                // With an input, refutable alternatives took the decision path.
                // Without one, every alternative declares the same bindings.
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
    pub(super) fn selected_place(
        &mut self,
        id: PatternId,
        base: Place,
        selection: Option<Projection>,
    ) -> Result<Place, Error> {
        let place = self.pattern_place(id, base, selection)?;
        if place != base
            && let Place::Projection(projection) = place
        {
            self.statement(
                self.facts.patterns.nodes[id.0 as usize].source.node,
                super::Action::Project(projection),
            )?;
        }
        Ok(place)
    }
    pub(super) fn pattern_place(
        &mut self,
        id: PatternId,
        base: Place,
        selection: Option<Projection>,
    ) -> Result<Place, Error> {
        let Some(selection) = selection else {
            return Ok(base);
        };
        let pattern = &self.facts.patterns.nodes[id.0 as usize];
        let node = pattern.source.node;
        let ty = pattern
            .ty
            .ok_or_else(|| self.error(node, "unsolved pattern type"))?;
        let projection = match selection {
            Projection::Field(field) => ProjectionKind::Field(field),
            Projection::Index(index) => ProjectionKind::ConstantIndex(index),
            Projection::Subarray { start, end } => ProjectionKind::Subarray { start, end },
            Projection::Subslice { .. } | Projection::FromEnd(_) => {
                return Err(self.error(
                    node,
                    "slice pattern execution requires region and loan proofs",
                ));
            }
        };
        self.projection(node, ty, base, projection)
            .map(Place::Projection)
    }
    pub(super) fn pattern_capture(
        &mut self,
        id: PatternId,
        place: Option<Place>,
    ) -> Result<(), Error> {
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
        let ty = self.facts.definitions[definition.0 as usize]
            .ok_or_else(|| self.error(node, "unsolved binding type"))?;
        let local = self.local(node, ty, LocalKind::Binding, Some(definition), mutable)?;
        if let Some(place) = place {
            let value = self.capture_value(id, place, ty)?;
            self.store(node, Place::Local(local), value)?;
        }
        Ok(())
    }
    pub(super) fn capture_value(
        &mut self,
        id: PatternId,
        place: Place,
        target: TypeId,
    ) -> Result<ValueId, Error> {
        let pattern = &self.facts.patterns.nodes[id.0 as usize];
        let node = pattern.source.node;
        let Kind::Binding { capture, .. } = pattern.kind else {
            return Err(self.error(node, "missing capture operation"));
        };
        let (ty, operation) = match capture {
            Capture::Value => (pattern.ty, Rvalue::Read(place)),
            Capture::Borrow { .. } => (
                self.facts.patterns.borrow_types.get(&id).copied(),
                Rvalue::Borrow {
                    place,
                    kind: crate::body::ir::BorrowKind::Immediate,
                },
            ),
        };
        let ty = ty.ok_or_else(|| self.error(node, "missing selected capture type"))?;
        let value = self.evaluate(node, ty, operation)?;
        if ty == target {
            Ok(value)
        } else {
            self.evaluate(node, target, Rvalue::Constrain(value))
        }
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
