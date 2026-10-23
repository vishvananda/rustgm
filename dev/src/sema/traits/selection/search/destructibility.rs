//! Ordinary Destruct is universal only for its actual validated source identity.
use super::{Action, Decision, Declarations, Frame, Search, inference};
use crate::{
    resolve::model::DefId,
    support::{bindings::LangItem, constness::Constness},
    types::infer,
};
impl Search<'_> {
    pub(super) fn source_destruct_result(
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
        let Some(declaration) = catalog.language(LangItem::Destruct) else {
            return Ok(None);
        };
        if DefId(item.0) != declaration.definition {
            return Ok(None);
        }
        catalog.validated_language(LangItem::Destruct)?;
        if !arguments.is_empty() {
            return Err("source Destruct has unexpected arguments".into());
        }
        if frame.deferred || frame.partial.is_some() || frame.selected.is_some() {
            return Ok(None);
        }
        if frame.goal.constness() != Some(Constness::Ordinary) {
            return Err("source const destruction eligibility is not implemented".into());
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
        let shape = self.query.builtins.check(facts, goal.ty)?;
        if self
            .query
            .builtins
            .take_trait_proof()
            .is_some_and(|p| !p.is_empty())
        {
            return Err(
                "source destruction formation requires retained trait prerequisites".into(),
            );
        }
        if !shape.well_formed {
            return Ok(Some(Action::Complete(Decision::Absent)));
        }
        Ok(Some(self.complete_source_builtin(frame, goal, false)))
    }
}
