//! Negative completion preserves the exact application and every unpaid scoped debt.
use super::{Decision, Declarations, Search, Source};
use crate::sema::traits::selection::refutation::{Closed, Conditional, Refutation};

impl Search<'_> {
    pub(super) fn negative_child(&self, decision: Decision) -> Result<Decision, String> {
        let Decision::Refuted(index) = decision else {
            return Ok(decision);
        };
        if !self.query.nodes[index].debt_free {
            return Err("conditional negative implementation requires scoped debt payment".into());
        }
        Ok(Decision::Absent)
    }

    pub(super) fn refutation(
        mut self,
        facts: &Declarations,
        root: usize,
        expected: super::Predicate,
    ) -> Result<Refutation, String> {
        let node = &self.query.nodes[root];
        if node.provisional_size || node.cycle_dependent {
            return Err(
                "negative implementation cannot rely on provisional or cyclic evidence".into(),
            );
        }
        let Some(Source::Implementation { owner, arguments }) = node.source else {
            return Err("negative completion has no actual implementation owner".into());
        };
        if !super::super::refutation::implementation_negative(facts, owner)? {
            return Err("negative completion disagrees with its actual owner".into());
        }
        let closed = Closed {
            goal: node.goal,
            owner,
            arguments,
            assumptions: self.caller.assumptions.clone(),
            environment: self.caller.environment.cloned(),
            availability: self.query.availability,
            first_region: self.caller.first_region,
        };
        let debt_free = node.debt_free;
        let end = node.end;
        let mut proof = std::mem::replace(
            &mut self.query.allocations,
            super::allocations::Allocations::new(1),
        )
        .finish(end);
        let mut sized = Vec::new();
        self.collect_graph(
            facts,
            root,
            Some((owner, arguments)),
            &mut proof,
            &mut sized,
        )?;
        let refutation = if debt_free {
            Refutation::DebtFree(Box::new(closed))
        } else {
            Refutation::Conditional(Box::new(Conditional {
                identity: closed,
                proof,
                sized,
            }))
        };
        refutation.validate(expected, &self.caller, self.query.availability)?;
        Ok(refutation)
    }
}
