//! Builtin Field selection preserves actual semantic fields and ordinary child proofs.
use super::{Action, Decision, Declarations, Frame, Goal, Search, inference};
use crate::{
    sema::compiler_items::field_protocol::{self, Readiness},
    support::bindings::LangItem,
};

impl Search<'_> {
    pub(super) fn source_field_goal(facts: &Declarations, goal: &Goal) -> bool {
        let Some((item, _)) = goal.trait_arguments() else {
            return false;
        };
        facts
            .source_bindings
            .as_deref()
            .and_then(|catalog| catalog.language(LangItem::Field))
            .is_some_and(|declaration| declaration.definition.0 == item.0)
    }

    pub(super) fn source_field_result(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Action>, String> {
        if !Self::source_field_goal(facts, &frame.goal)
            || frame.selected.is_some()
            || frame.partial.is_some()
            || frame.deferred
        {
            return Ok(None);
        }
        let goal = match frame
            .inputs
            .context
            .resolved_predicate(&mut facts.types, &frame.goal)
        {
            Ok(goal) => goal,
            Err(
                crate::types::infer::Error::Unresolved | crate::types::infer::Error::Normalization,
            ) => return Ok(Some(Action::Complete(Decision::Deferred))),
            Err(error) => return Err(inference(error)),
        };
        let action = match field_protocol::inspect(facts, goal)? {
            Readiness::Unrelated => return Ok(None),
            Readiness::Contracts(member) => {
                Action::Wait(super::Dependency::SourceResultContracts(member))
            }
            Readiness::Fields(item) => Action::Wait(super::Dependency::Fields(item)),
            Readiness::Deferred => Action::Complete(Decision::Deferred),
            Readiness::Absent => Action::Complete(Decision::Absent),
            Readiness::Ready(description) => {
                frame.require_nominal_formed(facts, goal.ty)?;
                let required = field_protocol::requirements(facts, goal, description)?;
                self.start_component_predicates(facts, frame, goal, &required)?
            }
        };
        Ok(Some(action))
    }
}
