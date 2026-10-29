//! Solved tracked calls demand real site data before any call publication commits.
use super::{Checker, Dependency, Error, NodeId, Requirement};
use crate::{
    sema::compiler_items::site_records, support::bindings::LangItem, syntax::ast::Kind,
    types::infer::Solved,
};

impl Checker<'_> {
    pub(super) fn caller_location_dependency(
        &mut self,
        solved: &Solved,
    ) -> Result<Option<Dependency>, Error> {
        if self.caller_data_deferred()? {
            return Ok(None);
        }
        let Some(catalog) = self.facts.source_bindings.as_deref() else {
            return Ok(None);
        };
        // Completed source data is an O(1) fast path; an absent declaration must
        // remain harmless until a real tracked call actually consumes it.
        let pending = if catalog.language(LangItem::PanicLocation).is_some() {
            let Some(owner) = site_records::requested_location(self.report, self.facts)? else {
                return Ok(None);
            };
            Some(owner)
        } else {
            None
        };
        let Some(node) = self.pending_caller_site(solved)? else {
            return Ok(None);
        };
        let owner = match pending {
            Some(owner) => owner,
            None => {
                site_records::requested_location(self.report, self.facts)?.ok_or_else(|| {
                    self.error(node, "caller-location readiness changed during query")
                })?
            }
        };
        Ok(Some(Dependency {
            requirement: Requirement::CallerLocation(owner),
            source: self.key(node),
        }))
    }

    fn pending_caller_site(&mut self, solved: &Solved) -> Result<Option<NodeId>, Error> {
        // Use touched vectors and solved terms, not unpublished method facts or
        // a scan of the retained active-call hash table's capacity.
        for index in 0..self.state.call_types.methods.len() {
            let (node, pending) = &self.state.call_types.methods[index];
            let node = *node;
            let [callable, actual, _] = pending.value_types(solved);
            self.caller_readiness_step(node)?;
            if !self.source_scalar_unary_type(node, actual)
                && !self.source_scalar_binary_solved(node, actual, solved)?
                && !self.source_scalar_comparison_solved(node, solved)?
                && self.requires_caller_site(node, callable)?
            {
                return Ok(Some(node));
            }
        }
        for index in 0..self.state.calls.len() {
            let (node, _) = self.state.calls[index];
            self.caller_readiness_step(node)?;
            if let Kind::Call { callee, .. } = self.kind(node) {
                let callable = solved.types[self.term(callee)?.0 as usize];
                if self.requires_caller_site(node, callable)? {
                    return Ok(Some(node));
                }
            }
        }
        for index in 0..self.state.call_types.try_plans.len() {
            let open = self.state.call_types.try_plans[index];
            self.caller_readiness_step(open.node)?;
            let calls = match open.kind {
                super::try_control::OpenKind::Block { call, .. } => [Some(call), None],
                super::try_control::OpenKind::Branch {
                    branch, residual, ..
                } => [Some(branch), Some(residual)],
            };
            for call in calls.into_iter().flatten() {
                let call = call.solved(solved).map_err(|e| self.error(open.node, e))?;
                if self.requires_caller_site(open.node, call.callable)? {
                    return Ok(Some(open.node));
                }
            }
        }
        Ok(None)
    }
    fn caller_readiness_step(&mut self, node: NodeId) -> Result<(), Error> {
        self.state.steps = self.state.steps.saturating_add(1);
        if self.state.steps > 1_000_000 {
            return Err(self.error(node, "body type work budget exceeded"));
        }
        Ok(())
    }
}
