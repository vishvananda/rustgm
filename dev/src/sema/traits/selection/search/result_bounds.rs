//! Declared object promises and proved projection results share scoped selection.
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
            .inputs
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
        frame
            .inputs
            .context
            .account_work(steps)
            .map_err(inference)?;
        let first = frame.available_region().max(self.query.allocations.floor());
        let selected = match rules.select_scoped(
            &mut facts.types,
            &mut frame.inputs.context,
            &frame.goal,
            first,
        ) {
            Ok(Selection::Proven(proof)) => proof,
            Ok(Selection::Absent) => return Ok(None),
            Ok(Selection::Deferred) | Err(infer::Error::Normalization) => {
                return Ok(Some(Action::Complete(Decision::Deferred)));
            }
            Err(error) => return Err(inference(error)),
        };
        self.query.reserve(frame.scope, selected.regions.clone())?;
        self.query.allocations.record(selected.regions.end);
        let goal = frame
            .inputs
            .context
            .resolved_predicate(&mut facts.types, &frame.goal)
            .map_err(inference)?;
        let index = self.query.push(
            Node {
                cycle_dependent: false,
                provisional_size: false,
                goal,
                source: Some(Source::result(&selected).map_err(inference)?),
                children: Vec::new(),
                first: frame.first,
                end: selected.regions.end,
                equalities: std::mem::take(&mut frame.inputs.context.region_equalities),
                outlives: Vec::new(),
                type_outlives: Vec::new(),
                sized: Vec::new(),
            },
            frame.scope,
            selected.regions,
        );
        // input_evidence attaches every parent normalization child before this
        // result can escape the proof stack, including all existential scopes.
        Ok(Some(Action::Complete(Decision::Proven(index))))
    }
}
