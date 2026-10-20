//! Nested normal forms wake only their dependent applications, keeping input proofs separate.
use super::{Checker, Error, NodeId, Pending, Query, ScopeId, Task, TermId, WatchId};

pub(super) enum Readiness {
    Waiting,
    Partial,
    Resolved,
}

impl Pending {
    fn input_scope(
        &mut self,
        parent: ScopeId,
        term: TermId,
        count: u32,
    ) -> Result<ScopeId, &'static str> {
        let data = self.data.get_or_insert_with(Default::default);
        if let Some(&scope) = data.input_scopes.get(&(parent, term)) {
            return Ok(scope);
        }
        let scope = data.scopes.scope(parent, count)?;
        data.input_scopes.insert((parent, term), scope);
        Ok(scope)
    }
}

impl Checker<'_> {
    pub(super) fn projection_readiness(
        &mut self,
        query: &mut Query,
        mut resolved: bool,
        watch: WatchId,
    ) -> Result<Readiness, Error> {
        let node = query.node;
        let term = match query.task {
            Task::Normalize { input, .. } | Task::Symbolic { input, .. } => input,
            Task::Contract(_) => return Ok(Readiness::Resolved),
        };
        // An inner normal form may erase holes from this application's inputs.
        // Its own retained query still owns those holes and their formation proof.
        if !resolved && query.input_normalizations {
            resolved = match self.state.infer.resolved(&mut self.facts.types, term) {
                Ok(_) => true,
                Err(crate::types::infer::Error::Unresolved) => false,
                Err(error) => return Err(self.error(node, error)),
            };
        }
        if !resolved
            && self
                .state
                .infer
                .trait_result(&self.facts.types, term)
                .map_err(|e| self.error(node, e))?
        {
            self.defer_input_normalizations(query, term, watch)?;
            return Ok(Readiness::Partial);
        }
        if matches!(query.task, Task::Normalize { .. })
            && !resolved
            && !self.projection_subject_known(node, term)?
        {
            let subject = self
                .state
                .infer
                .child(&self.facts.types, term, 0)
                .map_err(|e| self.error(node, e))?;
            if matches!(
                self.state.infer.head(&self.facts.types, subject),
                Err(crate::types::infer::Error::Normalization)
            ) {
                self.defer_input_normalizations(query, term, watch)?;
            }
            return Ok(Readiness::Waiting);
        }
        Ok(if resolved {
            Readiness::Resolved
        } else {
            Readiness::Partial
        })
    }

    pub(super) fn defer_input_normalizations(
        &mut self,
        query: &mut Query,
        term: TermId,
        watch: WatchId,
    ) -> Result<(), Error> {
        let node = query.node;
        let roots = self
            .state
            .infer
            .normalization_input_roots(
                &self.facts.types,
                term,
                query.scope,
                |parent, term, count| {
                    self.state
                        .projections
                        .input_scope(parent, term, count)
                        .map_err(|_| crate::types::infer::Error::InvalidIdentity)
                },
            )
            .map_err(|e| self.error(node, e))?;
        query.input_normalizations |= !roots.is_empty();
        for (scope, root) in roots {
            self.state
                .projections
                .data
                .get_or_insert_with(Default::default)
                .normalization_waiters
                .entry(root)
                .or_default()
                .insert(watch);
            self.defer_normalization(node, scope, root)?;
        }
        Ok(())
    }

    pub(super) fn published_normalization(
        &mut self,
        node: NodeId,
        root: TermId,
    ) -> Result<(), Error> {
        let waiters = self
            .state
            .projections
            .data
            .as_mut()
            .and_then(|data| data.normalization_waiters.remove(&root));
        // Stable ticket order preserves source-time region allocation when several
        // enclosing applications share one newly available input normal form.
        let mut waiters = waiters.into_iter().flatten().collect::<Vec<_>>();
        waiters.sort_unstable();
        for watch in waiters {
            self.state
                .infer
                .retry_watch(watch)
                .map_err(|e| self.error(node, e))?;
        }
        Ok(())
    }
}
