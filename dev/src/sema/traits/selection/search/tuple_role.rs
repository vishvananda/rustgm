//! Tuple evidence belongs to the validated source role, not its name or layout.
use super::{Action, Decision, Declarations, Frame, Search, inference};
use crate::{resolve::model::DefId, support::bindings::LangItem, types::infer};
impl Search<'_> {
    pub(super) fn source_tuple_result(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Action>, String> {
        let Some(catalog) = facts.source_bindings.as_deref() else {
            return Ok(None);
        };
        let Some((item, arguments)) = frame.goal.trait_arguments() else {
            return Ok(None);
        };
        let Some(declaration) = catalog.language(LangItem::Tuple) else {
            return Ok(None);
        };
        if DefId(item.0) != declaration.definition {
            return Ok(None);
        }
        catalog.validated_language(LangItem::Tuple)?;
        if !arguments.is_empty() {
            return Err("source Tuple has unexpected arguments".into());
        }
        if frame.deferred || frame.partial.is_some() || frame.selected.is_some() {
            return Ok(None);
        }
        let goal = match frame
            .inputs
            .context
            .resolved_predicate(&mut facts.types, &frame.goal)
        {
            Ok(goal) => goal,
            Err(infer::Error::Unresolved | infer::Error::Normalization) => {
                return Ok(Some(Action::Complete(Decision::Deferred)));
            }
            Err(error) => return Err(inference(error)),
        };
        if !crate::sema::compiler_items::tuple_predicate(facts, goal)? {
            return Ok(Some(Action::Complete(Decision::Absent)));
        }
        Ok(Some(self.complete_source_builtin(frame, goal, false)))
    }
}
