//! Selected receiver permissions and array precision reuse typed inference facts.
use super::{AccessKind, Checker, Collector, Error, NodeId, Use, Work};
use crate::{sema::model::AdjustmentKind, types::infer::Constructor};

#[derive(Clone, Copy)]
pub(super) enum ReferenceUse {
    Reference(crate::sema::model::ReferenceConversion, super::TermId),
    Value { mutable: bool },
}
/// Temporary transfer of the collector's two existing selection tables.
pub(super) struct ReferenceUses {
    pub(super) references: crate::support::hash::HashMap<NodeId, ReferenceUse>,
    pub(super) adjusted: crate::support::hash::HashSet<NodeId>,
}
pub(super) fn reference_uses(
    checker: &mut Checker<'_>,
    source: NodeId,
) -> Result<ReferenceUses, Error> {
    checker
        .state
        .infer
        .account_work(checker.state.adjustments.len())
        .map_err(|error| checker.error(source, error))?;
    let mut references = crate::support::hash::HashMap::default();
    let mut adjusted = crate::support::hash::HashSet::default();
    for (index, adjustment) in checker.state.adjustments.iter().enumerate() {
        // Keep resets in source order: a final contextual no-op cannot leave
        // an earlier synthetic dereference in the capture selection.
        if adjustment.reset {
            references.remove(&adjustment.node);
            adjusted.remove(&adjustment.node);
        }
        if let Some(&reference) = checker.state.reference_adjustments.get(&index) {
            references.insert(
                adjustment.node,
                ReferenceUse::Reference(reference.0, reference.1),
            );
        }
        if let Some(pending) = checker
            .state
            .value_reborrows
            .as_deref()
            .and_then(|p| p.get(&index))
        {
            references.insert(
                adjustment.node,
                ReferenceUse::Value {
                    mutable: pending.capture_mutability(),
                },
            );
        }
        if !matches!(
            adjustment.kind,
            AdjustmentKind::Equality
                | AdjustmentKind::Never
                | AdjustmentKind::ArrayToSlice
                | AdjustmentKind::Unsize
                | AdjustmentKind::Pointer
                | AdjustmentKind::FunctionPointer
        ) {
            adjusted.insert(adjustment.node);
        }
    }
    Ok(ReferenceUses {
        references,
        adjusted,
    })
}

impl Collector<'_> {
    pub(super) fn reference_use(
        &self,
        checker: &mut Checker<'_>,
        work: &Work,
    ) -> Result<Work, Error> {
        let selected = self
            .references
            .get(&work.node)
            .copied()
            .ok_or_else(|| checker.error(work.node, "missing capture borrow selection"))?;
        if work.dereferences != 0 || work.receiver.is_some() {
            return Err(checker.error(work.node, "conflicting capture borrow uses"));
        }
        // Both forms evaluate their actual source place, including wildcard
        // destinations. A value reborrow adds no reference dereference prefix.
        let (usage, dereferences) = match selected {
            ReferenceUse::Reference(conversion, borrow) => {
                let usage = match checker.head(work.node, borrow)? {
                    Some(Constructor::Reference { mutable: true, .. }) => Use::Mutable,
                    Some(Constructor::Reference { mutable: false, .. }) => Use::Shared,
                    _ => {
                        return Err(
                            checker.error(work.node, "missing capture reference borrow type")
                        );
                    }
                };
                (usage, conversion.dereferences)
            }
            ReferenceUse::Value { mutable } => {
                (if mutable { Use::Mutable } else { Use::Shared }, 0)
            }
        };
        Ok(Work {
            node: work.node,
            usage,
            dereferences,
            receiver: Some((work.node, AccessKind::Reference)),
        })
    }

    pub(super) fn call_receiver(
        &self,
        checker: &mut Checker<'_>,
        node: NodeId,
        callee: NodeId,
    ) -> Result<(Option<crate::support::call_traits::CallTrait>, u32), Error> {
        if let Some(selected) = self.calls.get(&node) {
            return Ok(*selected);
        }
        // Ordinary function items, constructors and function pointers already
        // have a checked call signature and no callable-trait receiver borrow.
        let term = checker.term(callee)?;
        if matches!(
            checker.head(node, term)?,
            Some(
                Constructor::Function { .. }
                    | Constructor::FunctionItem(..)
                    | Constructor::FunctionItemArgs(..)
            )
        ) {
            return Ok((None, 0));
        }
        Err(checker.error(node, "missing capture call receiver selection"))
    }

    pub(super) fn method(
        &mut self,
        checker: &mut Checker<'_>,
        node: NodeId,
        receiver: NodeId,
    ) -> Result<(), Error> {
        let selected = checker.field_capture_receiver(node)?;
        let (dereferences, borrow) = if let Some(selected) = selected {
            if selected.operand != receiver {
                return Err(
                    checker.error(node, "method capture changed its actual source receiver")
                );
            }
            (selected.dereferences, Some(selected.borrow))
        } else {
            self.methods
                .get(&node)
                .copied()
                .ok_or_else(|| checker.error(node, "missing capture method receiver selection"))?
        };
        let usage = if let Some(borrow) = borrow {
            match checker.head(node, borrow)? {
                Some(Constructor::Reference { mutable: true, .. }) => Use::Mutable,
                Some(Constructor::Reference { mutable: false, .. }) => Use::Shared,
                _ => return Err(checker.error(node, "missing capture method borrow type")),
            }
        } else {
            Use::Value
        };
        self.pending.push(Work {
            node: receiver,
            usage,
            dereferences,
            receiver: Some((node, AccessKind::Method)),
        });
        Ok(())
    }

    pub(super) fn index(
        &mut self,
        checker: &mut Checker<'_>,
        work: &Work,
        value: NodeId,
        index: NodeId,
    ) -> Result<(), Error> {
        let dereferences = checker
            .facts
            .autoderefs
            .get(&checker.key(work.node))
            .copied()
            .unwrap_or(0);
        let usage = if let Some((0, Some(borrow))) = self.methods.get(&work.node).copied() {
            match checker.head(work.node, borrow)? {
                Some(Constructor::Reference { mutable: true, .. }) => Use::Mutable,
                Some(Constructor::Reference { mutable: false, .. }) => Use::Shared,
                _ => return Err(checker.error(work.node, "missing selected index receiver borrow")),
            }
        } else {
            let mut term = checker.term(value)?;
            for _ in 0..dereferences {
                term = checker.child(work.node, term, 0)?;
            }
            if !matches!(
                checker.head(work.node, term)?,
                Some(
                    Constructor::Array(_)
                        | Constructor::ArrayTerm(_)
                        | Constructor::ArrayInfer(_)
                        | Constructor::Slice
                )
            ) {
                return Err(
                    checker.error(work.node, "missing typed capture index receiver selection")
                );
            }
            if matches!(work.usage, Use::Value) {
                Use::ValueOf(checker.term(work.node)?)
            } else {
                work.usage
            }
        };
        // Capture the selected receiver, never a method's invocation-time result.
        // The index remains its own ordinary use and is evaluated by the body CFG.
        self.pending.push(Work {
            node: value,
            usage,
            dereferences,
            receiver: Some((work.node, AccessKind::Index)),
        });
        self.push(index, Use::Value);
        Ok(())
    }

    pub(super) fn field(
        &mut self,
        checker: &mut Checker<'_>,
        node: NodeId,
        value: NodeId,
        usage: Use,
    ) -> Result<(), Error> {
        let Some(receiver) = checker.field_capture_receiver(node)? else {
            return self.projected(checker, node, value, usage);
        };
        let usage = match checker.head(node, receiver.borrow)? {
            Some(Constructor::Reference { mutable: true, .. }) => Use::Mutable,
            Some(Constructor::Reference { mutable: false, .. }) => Use::Shared,
            _ => return Err(checker.error(node, "field capture lost its actual receiver borrow")),
        };
        // Capture the actual call input. The result's fields and every Deref
        // effect remain in the ordinary checked closure body, evaluated on call.
        self.pending.push(Work {
            node: receiver.operand,
            usage,
            dereferences: receiver.dereferences,
            receiver: Some((node, AccessKind::Field)),
        });
        Ok(())
    }

    pub(super) fn projected(
        &mut self,
        checker: &mut Checker<'_>,
        node: NodeId,
        value: NodeId,
        usage: Use,
    ) -> Result<(), Error> {
        // A local projection is not itself a capture. Delay the Copy query until
        // traversal reaches an imported place, preserving the original leaf type
        // through every array/field boundary in the meantime.
        let usage = if matches!(usage, Use::Value) {
            Use::ValueOf(checker.term(node)?)
        } else {
            usage
        };
        self.push(value, usage);
        Ok(())
    }
}
