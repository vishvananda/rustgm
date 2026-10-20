//! Projected builtin premises use the same normalization stack and lifetime evidence.
use super::{Action, Decision, Declarations, Frame, Node, Search, inference};
use crate::types::{RegionId, TypeId, parameters::predicates::Bound};

#[derive(Default)]
pub(super) struct Requirements {
    pub cursor: usize,
    pub type_outlives: Vec<(TypeId, RegionId)>,
    pub sized: Vec<TypeId>,
}

impl Search<'_> {
    pub(super) fn complete_builtin(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Action, String> {
        let goal = match frame
            .inputs
            .context
            .resolved_predicate(&mut facts.types, &frame.goal)
        {
            Ok(goal) => goal,
            Err(crate::types::infer::Error::Unresolved) => {
                return Ok(Action::Complete(Decision::Deferred));
            }
            Err(error) => return Err(inference(error)),
        };
        let mut type_outlives = Vec::new();
        let mut sized = Vec::new();
        let mut provisional_size = false;
        if let Bound::Outlives(region) = goal.bound {
            type_outlives.push((goal.ty, region));
        } else {
            let valid = if goal.bound == Bound::Sized
                && (facts.types.contains_projected_sizes(goal.ty) == Some(true)
                    || !facts.types.nominal_sizes_sealed()
                        && facts.types.contains_nominals(goal.ty) == Some(true))
            {
                match self.storage_condition(facts, frame, goal.ty)? {
                    std::ops::ControlFlow::Break(action) => return Ok(action),
                    std::ops::ControlFlow::Continue((sized, provisional)) => {
                        provisional_size = provisional;
                        sized
                    }
                }
            } else {
                self.query.builtins.prove(facts, goal)?
            };
            if !valid {
                return Ok(Action::Complete(Decision::Absent));
            }
            if goal.bound == Bound::Sized && self.caller.retain_sized {
                sized.push(goal.ty);
            }
        }
        let children = frame
            .storage
            .as_mut()
            .map_or_else(Vec::new, |storage| std::mem::take(&mut storage.children));
        let node = self.query.push(
            Node {
                provisional_size,
                goal,
                source: Some(super::Source::Builtin),
                children,
                first: frame.first,
                end: frame.available_region(),
                equalities: std::mem::take(&mut frame.inputs.context.region_equalities),
                outlives: Vec::new(),
                type_outlives,
                sized,
            },
            frame.scope,
            frame.first..frame.first,
        );
        Ok(Action::Complete(Decision::Proven(node)))
    }
}
