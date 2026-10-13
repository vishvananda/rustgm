//! Retain packed-place obligations before unreachable syntax loses its CFG.
use super::{Checker, Error, Kind, NodeId, Type};
use crate::{
    sema::{
        model::ReferenceAlignment,
        patterns::{Capture, Kind as Pattern, PatternId},
    },
    types::TypeId,
};
use std::collections::HashMap;

#[derive(Default)]
pub(super) struct Pending {
    pub borrows: Vec<(NodeId, NodeId)>,
    pub patterns: Vec<(PatternId, Option<NodeId>)>,
    places: HashMap<NodeId, Option<u32>>,
    place_work: Vec<(NodeId, Option<u32>)>,
    pattern_work: Vec<(PatternId, Option<u32>)>,
    steps: usize,
}
fn capped(a: Option<u32>, b: Option<u32>) -> Option<u32> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
    }
}
impl Checker<'_> {
    fn alignment_step(&mut self, node: NodeId, count: usize) -> Result<(), Error> {
        self.alignment.steps = self
            .alignment
            .steps
            .checked_add(count)
            .filter(|n| *n <= 1_000_000)
            .ok_or_else(|| self.error(node, "reference alignment work budget exceeded"))?;
        Ok(())
    }
    fn packed_type(&self, ty: TypeId) -> Option<u32> {
        if let Some(Type::Adt(item, _)) = self.facts.types.kind(ty) {
            self.facts.aggregates.get(item).and_then(|d| d.repr.packed)
        } else {
            None
        }
    }
    fn packed_place(&mut self, mut node: NodeId) -> Result<Option<u32>, Error> {
        self.alignment.place_work.clear();
        let mut pack = loop {
            self.alignment_step(node, 1)?;
            if let Some(&pack) = self.alignment.places.get(&node) {
                break pack;
            }
            let (base, field) = match self.kind(node) {
                Kind::Field { value, .. } => (value, true),
                Kind::Index { value, .. } | Kind::Group(value) | Kind::Attributed { value, .. } => {
                    (value, false)
                }
                _ => break None,
            };
            let dereferences = self
                .facts
                .autoderefs
                .get(&self.key(node))
                .copied()
                .unwrap_or(0);
            let mut ty = self.ty(base)?;
            for _ in 0..dereferences {
                self.alignment_step(node, 1)?;
                let Some(Type::Reference { referent, .. }) = self.facts.types.kind(ty) else {
                    return Err(self.error(node, "invalid aligned place dereference"));
                };
                ty = referent;
            }
            let cap = if field { self.packed_type(ty) } else { None };
            self.alignment.place_work.push((node, cap));
            if dereferences != 0 {
                break None;
            }
            node = base;
        };
        while let Some((node, cap)) = self.alignment.place_work.pop() {
            pack = capped(pack, cap);
            self.alignment.places.insert(node, pack);
        }
        Ok(pack)
    }
    fn alignment_requirement(
        &mut self,
        source: NodeId,
        ty: TypeId,
        packed: Option<u32>,
        guard: bool,
    ) -> Result<(), Error> {
        if let Some(packed) = packed {
            self.alignment_step(source, 1)?;
            self.facts.reference_alignment.push(ReferenceAlignment {
                source: self.key(source),
                ty,
                packed,
                guard,
            });
        }
        Ok(())
    }
    fn pattern_alignment(
        &mut self,
        root: PatternId,
        packed: Option<u32>,
        guard: bool,
    ) -> Result<(), Error> {
        self.alignment.pattern_work.push((root, packed));
        while let Some((id, pack)) = self.alignment.pattern_work.pop() {
            let pattern = &self.facts.patterns.nodes[id.0 as usize];
            let source = pattern.source.node;
            let kind = pattern.kind;
            let ty = pattern
                .ty
                .ok_or_else(|| self.error(source, "untyped reference alignment pattern"))?;
            self.alignment_step(source, 1)?;
            match kind {
                Pattern::Binding {
                    capture,
                    subpattern,
                    ..
                } => {
                    if guard || matches!(capture, Capture::Borrow { .. }) {
                        self.alignment_requirement(source, ty, pack, guard)?;
                    }
                    if let Some(id) = subpattern {
                        self.alignment.pattern_work.push((id, pack));
                    }
                }
                Pattern::Dereference { pattern, .. } => {
                    self.alignment.pattern_work.push((pattern, None));
                }
                Pattern::Product { fields, .. } | Pattern::Slice { fields, .. } => {
                    let cap = capped(pack, self.packed_type(ty));
                    self.alignment_step(source, (fields.end - fields.start) as usize)?;
                    self.alignment.pattern_work.extend(
                        self.facts
                            .patterns
                            .fields(fields)
                            .iter()
                            .rev()
                            .map(|f| (f.pattern, cap)),
                    );
                }
                Pattern::Or(alternatives) => {
                    self.alignment_step(source, (alternatives.end - alternatives.start) as usize)?;
                    self.alignment.pattern_work.extend(
                        self.facts
                            .patterns
                            .alternatives(alternatives)
                            .iter()
                            .rev()
                            .map(|&id| (id, pack)),
                    );
                }
                _ => {}
            }
        }
        Ok(())
    }
    pub(super) fn publish_reference_alignment(&mut self) -> Result<(), Error> {
        for i in 0..self.alignment.borrows.len() {
            let (source, operand) = self.alignment.borrows[i];
            let packed = self.packed_place(operand)?;
            self.alignment_requirement(source, self.ty(operand)?, packed, false)?;
        }
        for i in 0..self.alignment.patterns.len() {
            let (id, input) = self.alignment.patterns[i];
            let packed = input.map(|n| self.packed_place(n)).transpose()?.flatten();
            self.pattern_alignment(id, packed, false)?;
        }
        for i in 0..self.matches.len() {
            let decision = &self.facts.patterns.decisions[self.matches[i] as usize];
            let (input, start, end) = (decision.scrutinee.node, decision.start, decision.end);
            let packed = self.packed_place(input)?;
            for index in start..end {
                let arm = self.facts.patterns.arms[index as usize];
                if let Some(pattern) = arm.pattern {
                    self.pattern_alignment(pattern, packed, arm.guard.is_some())?;
                }
            }
        }
        self.alignment.borrows.clear();
        self.alignment.patterns.clear();
        self.alignment.places.clear();
        self.alignment.steps = 0;
        Ok(())
    }
}
