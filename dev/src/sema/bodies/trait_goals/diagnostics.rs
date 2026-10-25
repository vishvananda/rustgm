//! Failed obligations preserve their typed source identity while choosing text.
use super::{Checker, Error, NodeId};
use crate::types::{infer::predicates::Predicate as Goal, parameters::predicates::Predicate};

impl Checker<'_> {
    pub(super) fn canonical_trait_failure(
        &self,
        node: NodeId,
        predicate: Predicate,
        fallback: &'static str,
    ) -> Error {
        match self
            .facts
            .custom_trait_failure(self.report, predicate, fallback)
        {
            Ok(Some(message)) => self.error(node, message),
            Ok(None) => self.error(node, fallback),
            Err(reason) => self.error(node, format!("{fallback}; diagnostic contract: {reason}")),
        }
    }
    pub(super) fn suspended_trait_failure(
        &mut self,
        node: NodeId,
        goal: &Goal,
        fallback: &'static str,
    ) -> Error {
        if !self.has_goal_diagnostic(goal) {
            return self.error(node, fallback);
        }
        let result = self
            .state
            .canonical_trait_inference
            .as_mut()
            .ok_or("diagnostic canonical inference context is missing")
            .and_then(|context| {
                context
                    .resolved_predicate(&mut self.facts.types, goal)
                    .map_err(|_| "diagnostic canonical inputs remain unresolved")
            });
        match result {
            Ok(predicate) => self.canonical_trait_failure(node, predicate, fallback),
            Err(reason) => self.error(node, format!("{fallback}; diagnostic contract: {reason}")),
        }
    }
    pub(super) fn call_trait_failure(
        &mut self,
        node: NodeId,
        index: usize,
        fallback: &'static str,
    ) -> Error {
        let goal = &self.state.call_types.obligations[index].1;
        if !self.has_goal_diagnostic(goal) {
            return self.error(node, fallback);
        }
        match self
            .state
            .infer
            .resolved_predicate(&mut self.facts.types, goal)
        {
            Ok(predicate) => self.canonical_trait_failure(node, predicate, fallback),
            Err(_) => self.error(
                node,
                format!("{fallback}; diagnostic contract: call inputs remain unresolved"),
            ),
        }
    }
    fn has_goal_diagnostic(&self, goal: &Goal) -> bool {
        goal.trait_arguments()
            .is_some_and(|(item, _)| self.facts.has_trait_diagnostic(item))
    }
}
