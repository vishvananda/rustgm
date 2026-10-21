//! Selected receiver permissions and array precision reuse typed inference facts.
use super::{Checker, Collector, Error, NodeId, Use, Work};
use crate::types::infer::Constructor;

impl Collector {
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
        // A stored synthetic descendant cannot reconstruct its unadjusted source
        // reference. That needs a distinct adjusted-access contract in the IR.
        if dereferences != 0 && self.places.get(checker, receiver)?.is_some() {
            return Err(checker.error(
                node,
                "adjusted capture method receivers are not implemented",
            ));
        }
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
        let term = checker.term(value)?;
        if !matches!(
            checker.head(work.node, term)?,
            Some(Constructor::Array(_) | Constructor::ArrayTerm(_) | Constructor::ArrayInfer(_))
        ) {
            return Err(checker.error(
                work.node,
                "adjusted or overloaded capture indexing is not implemented",
            ));
        }
        // The leaf's read/borrow determines the required mode, but arrays are
        // captured as a whole. The index expression has its own ordinary uses.
        self.projected(checker, work.node, value, work.usage)?;
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
