//! Selected receiver permissions and array precision reuse typed inference facts.
use super::{AccessKind, Checker, Collector, Error, NodeId, Use, Work};
use crate::types::infer::Constructor;

impl Collector<'_> {
    pub(super) fn reference_use(
        &self,
        checker: &mut Checker<'_>,
        work: &Work,
    ) -> Result<Work, Error> {
        let (conversion, borrow) = self.references.get(&work.node).copied().ok_or_else(|| {
            checker.error(work.node, "missing capture reference conversion selection")
        })?;
        if work.dereferences != 0 || work.receiver.is_some() || matches!(work.usage, Use::Mention) {
            return Err(checker.error(work.node, "conflicting capture reference conversion uses"));
        }
        let usage = match checker.head(work.node, borrow)? {
            Some(Constructor::Reference { mutable: true, .. }) => Use::Mutable,
            Some(Constructor::Reference { mutable: false, .. }) => Use::Shared,
            _ => return Err(checker.error(work.node, "missing capture reference borrow type")),
        };
        Ok(Work {
            node: work.node,
            usage,
            dereferences: conversion.dereferences,
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
        let (dereferences, borrow) = self
            .methods
            .get(&node)
            .copied()
            .ok_or_else(|| checker.error(node, "missing capture method receiver selection"))?;
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
