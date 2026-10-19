//! Search frames keep alternatives isolated and delay builtin conditions until equations settle.
use super::{Context, Decision, Declarations, Goal, Node, Predicate, Search, Source, inference};
use crate::{
    resolve::model::DefId,
    sema::traits::selection::instantiation::{self, Instance},
    types::{infer, lifetime_proof::ScopeId, parameters::predicates::Bound, shape_key::ShapeKey},
};

pub(super) struct Frame {
    pub normalization: Option<Box<super::normalization::Normalization>>,
    /// A suspended normalizer has proved these inputs in this search's scope.
    /// Its evidence must accompany any result derived without repeating that work.
    pub normalized_inputs: Option<NormalizedInputs>,
    pub inputs: infer::refinement::Refinement,
    pub goal: Goal,
    pub first: u32,
    pub scope: ScopeId,
    pub started: bool,
    pub key: Option<ShapeKey>,
    pub canonical: Option<Predicate>,
    pub cycles: usize,
    pub candidates: Vec<DefId>,
    pub pending_headers: Option<std::vec::IntoIter<DefId>>,
    pub cursor: usize,
    pub trial: Option<Trial>,
    pub selected: Option<usize>,
    pub deferred: bool,
    pub partial: Option<Box<Trial>>,
    pub inference: Inference,
    pub suspended_slot: Option<usize>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Inference {
    Disabled,
    Selecting,
    Committed,
}
pub(super) struct NormalizedInputs {
    pub proof: Option<usize>,
}
pub(super) struct Trial {
    pub inputs: infer::refinement::Refinement,
    pub goal: Goal,
    pub instance: Instance,
    pub first: u32,
    pub scope: ScopeId,
    pub end: u32,
    pub pending: Vec<usize>,
    pub cursor: usize,
    pub wakeups: super::wakeups::Wakeups,
    pub children: Vec<usize>,
    pub requirements: Option<Box<super::builtins::Requirements>>,
}
impl Frame {
    pub fn new(
        facts: &Declarations,
        source: &mut Context,
        goal: &Goal,
        first: u32,
    ) -> Result<Self, String> {
        let mut inputs = infer::refinement::Refinement::default();
        let goal = inputs
            .initialize_predicate(&facts.types, source, goal)
            .map_err(inference)?;
        Ok(Self {
            normalization: None,
            normalized_inputs: None,
            inputs,
            goal,
            first,
            scope: ScopeId::ROOT,
            started: false,
            key: None,
            canonical: None,
            cycles: 0,
            candidates: Vec::new(),
            pending_headers: None,
            cursor: 0,
            trial: None,
            selected: None,
            deferred: false,
            partial: None,
            inference: Inference::Disabled,
            suspended_slot: None,
        })
    }
}
impl Trial {
    pub fn new(
        facts: &mut Declarations,
        frame: &mut Frame,
        owner: DefId,
        floor: u32,
    ) -> Result<Self, String> {
        let mut inputs = infer::refinement::Refinement::default();
        let goal = inputs
            .initialize_predicate(&facts.types, &mut frame.inputs.context, &frame.goal)
            .map_err(inference)?;
        let first = frame.available_region().max(floor);
        let instance = instantiation::instantiate(facts, &mut inputs.context, owner, first)?;
        let end = first
            + u32::try_from(instance.regions.len()).map_err(|_| "impl region count overflow")?;
        let pending = instance
            .goals
            .iter()
            .enumerate()
            .filter_map(|(index, goal)| {
                (goal.trait_arguments().is_some() || goal.projection_result().is_some())
                    .then_some(index)
            })
            .collect();
        Ok(Self {
            inputs,
            goal,
            instance,
            first,
            scope: frame.scope,
            end,
            pending,
            cursor: 0,
            wakeups: super::wakeups::Wakeups::default(),
            children: Vec::new(),
            requirements: None,
        })
    }
}
impl Search<'_> {
    pub(super) fn finish(
        &mut self,
        facts: &mut Declarations,
        trial: &mut Trial,
    ) -> Result<Option<Decision>, String> {
        let mut requirements = trial.requirements.take().map(|r| *r).unwrap_or_default();
        while let Some(goal) = trial.instance.goals.get(requirements.cursor) {
            let index = requirements.cursor;
            requirements.cursor += 1;
            let Some(bound) = goal.builtin_bound() else {
                continue;
            };
            let ty = match trial
                .inputs
                .context
                .resolved(&mut facts.types, goal.subject())
            {
                Ok(ty) => ty,
                Err(infer::Error::Unresolved) => {
                    requirements.cursor = index;
                    trial.requirements = Some(Box::new(requirements));
                    return Ok(Some(Decision::Deferred));
                }
                Err(error) => return Err(inference(error)),
            };
            if facts
                .types
                .contains_projections(ty)
                .ok_or("missing builtin subject")?
            {
                trial.pending.push(index);
                trial.requirements = Some(Box::new(requirements));
                return Ok(None);
            }
            if let Bound::Outlives(region) = bound {
                requirements.type_outlives.push((ty, region));
            } else if !self.query.builtins.prove(facts, Predicate { ty, bound })? {
                return Ok(Some(Decision::Absent));
            } else if bound == Bound::Sized && self.caller.retain_sized {
                requirements.sized.push(ty);
            }
        }
        let (goal, arguments) = match trial.resolved_parent(facts) {
            Ok(resolved) => resolved,
            Err(infer::Error::Unresolved) => {
                trial.requirements = Some(Box::new(requirements));
                return Ok(Some(Decision::Deferred));
            }
            Err(error) => return Err(inference(error)),
        };
        if !self.conditions(facts, trial, arguments)? {
            trial.requirements = Some(Box::new(requirements));
            return Ok(None);
        }
        let owned_end = trial.first
            + u32::try_from(trial.instance.regions.len())
                .map_err(|_| "impl region count overflow")?;
        let node = self.query.push(
            Node {
                goal,
                source: Some(Source::Implementation {
                    owner: trial.instance.owner,
                    arguments,
                }),
                children: std::mem::take(&mut trial.children),
                first: trial.first,
                end: trial.end,
                equalities: std::mem::take(&mut trial.inputs.context.region_equalities),
                outlives: std::mem::take(&mut trial.instance.outlives),
                type_outlives: requirements.type_outlives,
                sized: requirements.sized,
            },
            trial.scope,
            trial.first..owned_end,
        );
        Ok(Some(Decision::Proven(node)))
    }
}
