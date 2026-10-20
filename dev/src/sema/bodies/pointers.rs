//! Raw coercions erase the outer reference loan, retaining pointee obligations.
use super::{Checker, Constructor, Error, NodeId, TermId};
use crate::{
    resolve::unit::NodeKey,
    sema::model::{AdjustmentKind, BodyRegionOrigin, Coercion, ReferenceConversion},
    types::{Kind as Type, TypeId, infer::Solved},
};

pub(super) struct Pending {
    pub view: TermId,
    pub intermediate: TermId,
}

impl Checker<'_> {
    pub(super) fn pointer_storage(&self, solved: &Solved) -> Vec<TypeId> {
        if self.state.pointer_adjustments.is_empty() {
            return Vec::new();
        }
        self.state
            .adjustments
            .iter()
            .enumerate()
            .filter_map(|(index, adjustment)| {
                if adjustment.kind != AdjustmentKind::PointerReborrow {
                    return None;
                }
                let (_, borrow) = self.state.reference_adjustments.get(&index)?;
                match self.facts.types.kind(solved.types[borrow.0 as usize]) {
                    Some(Type::Reference { referent, .. }) => Some(referent),
                    _ => None,
                }
            })
            .collect()
    }
    pub(super) fn publish_pointer_adjustments(&mut self, solved: &Solved, body: NodeKey) {
        if self.state.pointer_adjustments.is_empty() {
            return;
        }
        for (index, adjustment) in self.state.adjustments.iter().enumerate() {
            if !self.state.pointer_adjustments.contains_key(&index) {
                continue;
            }
            let source = self.key(adjustment.node);
            if let Some((conversion, borrow)) = self.state.reference_adjustments.get(&index) {
                // Only the immediate reborrow inherits the original reference ancestry.
                // The resulting raw pointer keeps pointee regions, not this outer loan.
                self.facts.retain_coercion(Coercion {
                    body,
                    source,
                    actual: solved.types[adjustment.actual.0 as usize],
                    expected: solved.types[borrow.0 as usize],
                    kind: AdjustmentKind::Reborrow,
                    reference: Some(*conversion),
                });
            }
        }
    }
    pub(super) fn pointer_coercion(
        &mut self,
        node: NodeId,
        actual: TermId,
        expected: TermId,
    ) -> Result<(), Error> {
        let referent = self.child(node, actual, 0)?;
        let target = self.child(node, expected, 0)?;
        let unsizing = self.unsizing_referent(node, referent, target)?;
        let converted = unsizing.as_ref().map_or(referent, |pending| pending.view);
        self.apply_pointer_coercion(node, actual, expected, converted, unsizing)
    }
    pub(super) fn apply_pointer_coercion(
        &mut self,
        node: NodeId,
        actual: TermId,
        expected: TermId,
        converted: TermId,
        mut unsizing: Option<super::unsizing::Pending>,
    ) -> Result<(), Error> {
        let Some(Constructor::Pointer { mutable }) = self.head(node, expected)? else {
            return Err(self.error(node, "missing raw pointer coercion target"));
        };
        let source = self.head(node, actual)?;
        let Some(
            Constructor::Reference {
                mutable: source_mutable,
                ..
            }
            | Constructor::Pointer {
                mutable: source_mutable,
            },
        ) = source
        else {
            return Err(self.error(node, "raw pointer coercion requires a pointer or reference"));
        };
        if mutable && !source_mutable {
            return Err(self.error(node, "raw pointer coercion cannot strengthen mutability"));
        }
        let referent = self.child(node, actual, 0)?;
        let head = Constructor::Pointer { mutable };
        let view = self
            .state
            .infer
            .construct(head, &[converted])
            .map_err(|e| self.error(node, e))?;
        self.callable_shape(node, view, expected)?;
        if source == Some(head) && unsizing.is_none() {
            return self.record_adjustment(node, expected, AdjustmentKind::Equality);
        }
        if (converted == referent || unsizing.is_some())
            && self.propagate_reference(node, expected, true)?
        {
            return Ok(());
        }
        let intermediate = self
            .state
            .infer
            .construct(head, &[referent])
            .map_err(|e| self.error(node, e))?;
        let index = self.state.adjustments.len();
        let kind = if matches!(source, Some(Constructor::Reference { mutable: true, .. })) {
            let region = self.body_region(node, BodyRegionOrigin::Inference)?;
            let borrow = self
                .state
                .infer
                .construct(Constructor::Reference { mutable, region }, &[referent])
                .map_err(|e| self.error(node, e))?;
            self.state.reference_adjustments.insert(
                index,
                (
                    ReferenceConversion {
                        dereferences: 1,
                        unsize: false,
                        two_phase: false,
                    },
                    borrow,
                ),
            );
            AdjustmentKind::PointerReborrow
        } else {
            AdjustmentKind::Pointer
        };
        if let Some(pending) = &mut unsizing {
            self.prepare_unsizing_relations(node, pending)?;
            pending.view = view;
        }
        if let Some(pending) = unsizing {
            self.state.unsizing_adjustments.insert(index, pending);
        }
        self.state
            .pointer_adjustments
            .insert(index, Pending { view, intermediate });
        self.record_adjustment(node, expected, kind)
    }
    pub(super) fn pointer_join(
        &mut self,
        node: NodeId,
        a: TermId,
        b: TermId,
    ) -> Result<TermId, Error> {
        let mutable = matches!(
            self.head(node, a)?,
            Some(Constructor::Pointer { mutable: true })
        ) && matches!(
            self.head(node, b)?,
            Some(Constructor::Pointer { mutable: true })
        );
        let left = self.child(node, a, 0)?;
        let right = self.child(node, b, 0)?;
        self.callable_shape(node, left, right)?;
        let result = self
            .state
            .infer
            .construct(Constructor::Pointer { mutable }, &[left])
            .map_err(|e| self.error(node, e))?;
        self.inferred_regions(node, result)
    }
}
