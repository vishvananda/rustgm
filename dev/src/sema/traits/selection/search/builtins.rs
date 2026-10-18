//! Projected builtin premises use the same normalization stack and lifetime evidence.
use super::{Decision, Declarations, Frame, Node, Search, inference};
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
    ) -> Result<Decision, String> {
        let goal = match frame
            .context
            .resolved_predicate(&mut facts.types, &frame.goal)
        {
            Ok(goal) => goal,
            Err(crate::types::infer::Error::Unresolved) => return Ok(Decision::Deferred),
            Err(error) => return Err(inference(error)),
        };
        let mut type_outlives = Vec::new();
        let mut sized = Vec::new();
        if let Bound::Outlives(region) = goal.bound {
            type_outlives.push((goal.ty, region));
        } else if !self.query.builtins.prove(facts, goal)? {
            return Ok(Decision::Absent);
        } else if goal.bound == Bound::Sized && self.caller.retain_sized {
            sized.push(goal.ty);
        }
        let node = self.query.push(
            Node {
                goal,
                source: Some(super::Source::Builtin),
                children: Vec::new(),
                first: frame.first,
                end: frame.available_region(),
                equalities: std::mem::take(&mut frame.context.region_equalities),
                outlives: Vec::new(),
                type_outlives,
                sized,
            },
            frame.scope,
            frame.first..frame.first,
        );
        Ok(Decision::Proven(node))
    }
}
