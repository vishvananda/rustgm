//! Normalized projected fields require equality independently of outer-reference subtyping.
use super::{Checker, Constructor, Error, NodeId, Pending};
use crate::{
    resolve::unit::NodeKey,
    sema::model::{AdjustmentKind, Coercion},
    types::{infer::Solved, substitute::Bound},
};

impl Checker<'_> {
    pub(in crate::sema::bodies) fn prepare_unsizing_relations(
        &mut self,
        node: NodeId,
        pending: &mut Pending,
    ) -> Result<(), Error> {
        if pending.relations.is_empty() {
            return Ok(());
        }
        // A mutable-pointer proof view is invariant in its child and introduces
        // no reference lifetime. This is proof transport, never a runtime pointer.
        // Sharing the outer reference cannot weaken a projected field's equality.
        let invariant = Constructor::Pointer { mutable: true };
        for (left, right) in &mut pending.relations {
            *left = self
                .state
                .infer
                .construct(invariant, &[*left])
                .map_err(|e| self.error(node, e))?;
            *right = self
                .state
                .infer
                .construct(invariant, &[*right])
                .map_err(|e| self.error(node, e))?;
        }
        Ok(())
    }
    pub(in crate::sema::bodies) fn freshen_unsizing_relations(
        &mut self,
        node: NodeId,
        index: usize,
    ) -> Result<(), Error> {
        let Some(pending) = self.state.unsizing_adjustments.get_mut(&index) else {
            return Ok(());
        };
        let mut relations = std::mem::take(&mut pending.relations);
        for (left, right) in &mut relations {
            *left = self.inferred_regions(node, *left)?;
            *right = self.inferred_regions(node, *right)?;
        }
        self.state
            .unsizing_adjustments
            .get_mut(&index)
            .ok_or_else(|| Error::at(self.state.file, 0, "missing unsizing relations"))?
            .relations = relations;
        Ok(())
    }
    pub(in crate::sema::bodies) fn publish_unsizing_relations(
        &mut self,
        solved: &Solved,
        body: NodeKey,
        mut defining: Option<&mut Bound>,
    ) -> Result<(), Error> {
        // Syntax adjustment order is stable; the sparse map's iteration order is not.
        for (index, adjustment) in self.state.adjustments.iter().enumerate() {
            let Some(pending) = self.state.unsizing_adjustments.get(&index) else {
                continue;
            };
            let source = self.key(adjustment.node);
            for &(left, right) in &pending.relations {
                let mut actual = solved.types[left.0 as usize];
                let mut expected = solved.types[right.0 as usize];
                if let Some(fold) = &mut defining {
                    actual = fold
                        .apply(&mut self.facts.types, actual)
                        .map_err(|e| Error::at(source.file, 0, e))?;
                    expected = fold
                        .apply(&mut self.facts.types, expected)
                        .map_err(|e| Error::at(source.file, 0, e))?;
                }
                self.facts.retain_coercion(Coercion {
                    body,
                    source,
                    actual,
                    expected,
                    kind: AdjustmentKind::Equality,
                    reference: None,
                });
            }
        }
        Ok(())
    }
}
