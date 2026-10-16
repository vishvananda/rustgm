//! Demand result rules only after normalization proved the original parent application.
use super::{Action, Decision, Declarations, Frame, Node, Search, Source, inference};
use crate::{
    sema::traits::{assumptions::Selection, results},
    types::infer,
};

impl Search<'_> {
    pub(super) fn derive_result(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Action>, String> {
        let ty = match frame
            .context
            .resolved(&mut facts.types, frame.goal.subject())
        {
            Ok(ty) => ty,
            Err(infer::Error::Unresolved) => return Ok(None),
            Err(error) => return Err(inference(error)),
        };
        let (item, _) = frame
            .goal
            .trait_arguments()
            .ok_or("missing result query trait")?;
        let Some((rules, steps)) = results::traits(facts, ty, item)? else {
            return Ok(None);
        };
        frame.context.account_work(steps).map_err(inference)?;
        let selected = match rules.select(&mut facts.types, &mut frame.context, &frame.goal) {
            Ok(Selection::Proven(proof)) => proof.assumption(),
            Ok(Selection::Absent) => return Ok(None),
            Ok(Selection::Deferred) | Err(infer::Error::Normalization) => {
                return Ok(Some(Action::Complete(Decision::Deferred)));
            }
            Err(error) => return Err(inference(error)),
        };
        let goal = frame
            .context
            .resolved_predicate(&mut facts.types, &frame.goal)
            .map_err(inference)?;
        let index = self.nodes.len();
        self.nodes.push(Node {
            goal,
            source: Some(Source::ResultBound(selected)),
            children: Vec::new(),
            first: frame.first,
            end: frame.available_region(),
            equalities: std::mem::take(&mut frame.context.region_equalities),
            outlives: Vec::new(),
            type_outlives: Vec::new(),
            sized: Vec::new(),
        });
        // input_evidence attaches every parent normalization child before this
        // result can escape the proof stack, including all existential scopes.
        Ok(Some(Action::Complete(Decision::Proven(index))))
    }
}
