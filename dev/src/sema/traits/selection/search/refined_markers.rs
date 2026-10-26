//! Pattern marker evidence requires the exact source trait on its actual base.
use super::{Action, Declarations, Frame, Goal, Search, inference};
use crate::types::infer::Constructor;
impl Search<'_> {
    pub(super) fn source_refinement_marker_goal(
        facts: &Declarations,
        frame: &mut Frame,
    ) -> Result<bool, String> {
        if !super::cloning::source_copying_role(facts, &frame.goal)? {
            return Ok(false);
        }
        let head = match frame
            .inputs
            .context
            .head(&facts.types, frame.goal.subject())
        {
            Ok(head) => head,
            Err(crate::types::infer::Error::Normalization) => return Ok(false),
            Err(error) => return Err(inference(error)),
        };
        Ok(matches!(head, Some(Constructor::Pattern(_))))
    }
    pub(super) fn source_refinement_marker_result(
        facts: &Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Action>, String> {
        if frame.deferred
            || frame.partial.is_some()
            || frame.selected.is_some()
            || !Self::source_refinement_marker_goal(facts, frame)?
        {
            return Ok(None);
        }
        frame.inputs.context.account_work(1).map_err(inference)?;
        let base = frame
            .inputs
            .context
            .child(&facts.types, frame.goal.subject(), 0)
            .map_err(inference)?;
        let (item, arguments) = frame
            .goal
            .trait_arguments()
            .ok_or("pattern marker lost its actual trait")?;
        if !arguments.is_empty() {
            return Err("pattern marker has unexpected source trait arguments".into());
        }
        // This descends through the same query/caller and preserves every child debt.
        Ok(Some(Self::start_refinement(
            frame,
            Goal::trait_application(base, item, Vec::new()),
        )))
    }
}
