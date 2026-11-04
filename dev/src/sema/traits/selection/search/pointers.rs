//! Function-pointer evidence preserves the real role and caller's current inference.
use super::{Action, Decision, Declarations, Frame, Search, inference};
use crate::{
    support::{bindings::LangItem, constness::Constness},
    types::{Kind, infer},
};
impl Search<'_> {
    pub(super) fn source_fn_pointer_goal(
        facts: &Declarations,
        goal: &super::Goal,
    ) -> Result<bool, String> {
        let Some((item, arguments)) = goal.trait_arguments() else {
            return Ok(false);
        };
        let Some(catalog) = facts.source_bindings.as_deref() else {
            return Ok(false);
        };
        let Some(declaration) = catalog.language(LangItem::FnPtrTrait) else {
            return Ok(false);
        };
        if declaration.definition.0 != item.0 {
            return Ok(false);
        }
        catalog.validated_language(LangItem::FnPtrTrait)?;
        if !arguments.is_empty() {
            return Err("source FnPtr has unexpected trait arguments".into());
        }
        Ok(goal.constness() == Some(Constness::Ordinary))
    }
    pub(super) fn source_fn_pointer_result(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Action>, String> {
        if !Self::source_fn_pointer_goal(facts, &frame.goal)? {
            return Ok(None);
        }
        if frame.selected.is_some()
            || frame.deferred
            || frame.partial.is_some()
            || frame.goal.constness() != Some(Constness::Ordinary)
        {
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
        if !matches!(facts.types.kind(goal.ty), Some(Kind::Function { .. })) {
            return Ok(Some(Action::Complete(Decision::Absent)));
        }
        frame.require_nominal_formed(facts, goal.ty)?;
        Ok(Some(self.complete_source_builtin(frame, goal, false)))
    }
}
