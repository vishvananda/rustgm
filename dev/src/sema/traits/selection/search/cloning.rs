//! Marker premises check abstract Clone applications; they never select a method body.
use super::{Action, Decision, Declarations, Frame, Node, Search, Source, inference};
use crate::types::parameters::predicates::{Bound, Predicate};
impl Search<'_> {
    pub(super) fn clone_marker_result(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Action>, String> {
        let Some(canonical) = facts.cloning.as_deref().map(|r| r.canonical) else {
            return Ok(None);
        };
        let Some((item, arguments)) = frame.goal.trait_arguments() else {
            return Ok(None);
        };
        if item.0 != canonical.definition.0
            || !arguments.is_empty()
            || frame.selected.is_some()
            || frame.deferred
            || frame.partial.is_some()
        {
            return Ok(None);
        }
        let goal = match frame
            .inputs
            .context
            .resolved_predicate(&mut facts.types, &frame.goal)
        {
            Ok(goal) => goal,
            Err(crate::types::infer::Error::Unresolved) => {
                return Ok(Some(Action::Complete(Decision::Deferred)));
            }
            Err(error) => return Err(inference(error)),
        };
        // All real candidates and their header waits were handled first. This
        // result carries no implementation identity; concrete dispatch rejects it.
        for bound in [Bound::Clone, Bound::Sized] {
            if !self
                .query
                .builtins
                .prove(facts, Predicate { ty: goal.ty, bound })?
            {
                return Ok(Some(Action::Complete(Decision::Absent)));
            }
        }
        let end = frame.available_region();
        let node = self.query.push(
            Node {
                cycle_dependent: false,
                provisional_size: false,
                goal,
                source: Some(Source::Builtin),
                children: Vec::new(),
                first: frame.first,
                end,
                equalities: std::mem::take(&mut frame.inputs.context.region_equalities),
                outlives: Vec::new(),
                type_outlives: Vec::new(),
                sized: Vec::new(),
            },
            frame.scope,
            end..end,
        );
        Ok(Some(Action::Complete(Decision::Proven(node))))
    }
}
