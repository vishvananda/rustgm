//! Failed obligations preserve their typed source identity while choosing text.
use super::{Checker, Error, NodeId};
use crate::types::{infer::predicates::Predicate as Goal, parameters::predicates::Predicate};
use crate::{lex::token::Punctuation, sema::compiler_items::string_role, syntax::ast::Kind};

enum GoalDiagnostic {
    Authored,
    StringConcatenation,
}

impl Checker<'_> {
    pub(in crate::sema::bodies) fn projection_selection_failure(
        &mut self,
        node: NodeId,
        fallback: &str,
    ) -> Error {
        if fallback == crate::sema::projections::MISSING_IMPLEMENTATION
            && let Kind::Binary {
                op: Punctuation::Plus,
                left,
                right,
            } = self.kind(node)
            && let (Ok(left), Ok(right)) = (self.term(left), self.term(right))
            && let Ok(left) = self.state.infer.resolved(&mut self.facts.types, left)
            && let Ok(right) = self.state.infer.resolved(&mut self.facts.types, right)
            && string_role::borrowed_types(self.facts, left, right)
        {
            return self.error(
                node,
                "string concatenation requires an owned `String` on the left",
            );
        }
        self.error(node, fallback)
    }

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
            Ok(None)
                if self.is_addition(node)
                    && string_role::borrowed_operands(self.facts, predicate) =>
            {
                self.error(
                    node,
                    "string concatenation requires an owned `String` on the left",
                )
            }
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
        let Some(diagnostic) = self.goal_diagnostic(node, goal) else {
            return self.error(node, fallback);
        };
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
            Err(reason) if matches!(diagnostic, GoalDiagnostic::Authored) => {
                self.error(node, format!("{fallback}; diagnostic contract: {reason}"))
            }
            Err(_) => self.error(node, fallback),
        }
    }
    pub(super) fn call_trait_failure(
        &mut self,
        node: NodeId,
        index: usize,
        fallback: &'static str,
    ) -> Error {
        let goal = &self.state.call_types.obligations[index].1;
        let Some(diagnostic) = self.goal_diagnostic(node, goal) else {
            return self.error(node, fallback);
        };
        match self
            .state
            .infer
            .resolved_predicate(&mut self.facts.types, goal)
        {
            Ok(predicate) => self.canonical_trait_failure(node, predicate, fallback),
            Err(_) if matches!(diagnostic, GoalDiagnostic::Authored) => self.error(
                node,
                format!("{fallback}; diagnostic contract: call inputs remain unresolved"),
            ),
            Err(_) => self.error(node, fallback),
        }
    }
    fn goal_diagnostic(&self, node: NodeId, goal: &Goal) -> Option<GoalDiagnostic> {
        let (item, _) = goal.trait_arguments()?;
        if self.facts.has_trait_diagnostic(item) {
            Some(GoalDiagnostic::Authored)
        } else {
            (self.is_addition(node) && string_role::addition(self.facts, item).is_some())
                .then_some(GoalDiagnostic::StringConcatenation)
        }
    }
    fn is_addition(&self, node: NodeId) -> bool {
        matches!(
            self.kind(node),
            Kind::Binary {
                op: Punctuation::Plus,
                ..
            }
        )
    }
}
