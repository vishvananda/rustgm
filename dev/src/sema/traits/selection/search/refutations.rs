//! Negative completion preserves the exact application and every unpaid scoped debt.
use super::{Decision, Declarations, Search, Source};
use crate::sema::traits::selection::refutation::{Closed, Conditional, Refutation};

/// Only this module can bind a completed payment to its same extracted graph.
pub(in crate::sema) struct Paid(Box<Conditional>);
impl Paid {
    pub(in crate::sema::traits::selection) fn identity(&self) -> &Closed {
        &self.0.identity
    }
}
#[cfg(test)]
const _: () = assert!(size_of::<Paid>() == size_of::<Box<Conditional>>());

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

    /// A completed child is an ordinary positive obligation of its parent,
    /// and rustc 1.98 never answers a positive goal with a negative impl. A
    /// child frame refutes only when it proved and deferred no positive
    /// candidate, so an ordinary trait's refuted child is simply unproven,
    /// whatever its scoped debts: `[I]: Iterator` under core's
    /// `impl<T> !Iterator for [T]` inside `Box<[I], A>: Iterator`. The
    /// intercrate check still asks whether a downstream crate could prove
    /// it; a negated where clause is proven outside intercrate mode, where
    /// a positive candidate's unproven child is just absent. An auto trait's
    /// refuted child keeps its debt payment.
    pub(super) fn positive_child(
        &self,
        facts: &mut Declarations,
        child: &mut super::Frame,
        decision: Decision,
    ) -> Result<Decision, String> {
        if !matches!(decision, Decision::Refuted(_))
            || matches!(
                child.goal.builtin_bound(),
                Some(crate::types::parameters::predicates::Bound::Auto(_))
            )
            || Self::source_auto_goal(facts, &child.goal)?
        {
            return Ok(decision);
        }
        if matches!(
            self.query.purpose,
            crate::sema::traits::implementations::Purpose::NegativeCoherence(_)
        ) {
            return Ok(Decision::Absent);
        }
        self.coherence_absence(facts, child, Decision::Absent)
    }

    pub(super) fn accept_negative(
        &mut self,
        facts: &Declarations,
        decision: Decision,
    ) -> Result<Decision, String> {
        if let Ok(result) = self.negative_child(decision) {
            return Ok(result);
        }
        let Decision::Refuted(index) = decision else {
            return Err("invalid negative completion".into());
        };
        if !self.pay_negative_equalities(facts, index)? {
            return Err("conditional negative implementation requires scoped debt payment".into());
        }
        Ok(Decision::Absent)
    }

    pub(super) fn paid_refutation(
        &mut self,
        facts: &mut Declarations,
        context: &mut super::Context,
        root: usize,
        expected: super::Predicate,
    ) -> Result<Refutation, String> {
        let before = self.query.steps;
        let nested_before = self.query.nested;
        let paid = self.pay_negative_equalities(facts, root);
        let charged = context.account_work(self.query.steps.saturating_sub(before));
        let nested =
            context.account_nested(self.query.nested.saturating_sub(nested_before) as usize);
        let paid = paid?;
        charged.map_err(super::inference)?;
        nested.map_err(super::inference)?;
        let result =
            self.pay_extraction(context, |search| search.refutation(facts, root, expected))?;
        Ok(match (paid, result) {
            (true, Refutation::Conditional(proof)) => Refutation::Equalities(Paid(proof)),
            (_, result) => result,
        })
    }

    pub(super) fn refutation(
        &mut self,
        facts: &mut Declarations,
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
        let mut meta_sized = Vec::new();
        self.collect_graph(
            facts,
            ([root], super::GraphTraversal::Ordinary),
            Some((owner, arguments)),
            &mut proof,
            &mut sized,
            Some(&mut meta_sized),
        )?;
        let refutation = if debt_free {
            Refutation::DebtFree(Box::new(closed))
        } else {
            Refutation::Conditional(Box::new(Conditional {
                identity: closed,
                proof,
                sized,
                meta_sized,
            }))
        };
        refutation.validate(expected, &self.caller, self.query.availability)?;
        Ok(refutation)
    }
}
