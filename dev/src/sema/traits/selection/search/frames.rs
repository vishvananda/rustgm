//! Search frames keep alternatives isolated and delay builtin conditions until equations settle.
use super::{Context, Decision, Declarations, Goal, Node, Predicate, Search, Source, inference};
use crate::{
    resolve::model::DefId,
    sema::traits::selection::instantiation::{self, Instance},
    types::{infer, parameters::predicates::Bound, shape_key::ShapeKey},
};

pub(super) struct Frame {
    pub normalization: Option<Box<super::normalization::Normalization>>,
    /// A suspended normalizer has proved these inputs in this search's scope.
    /// Its evidence must accompany any result derived without repeating that work.
    pub normalized_inputs: Option<NormalizedInputs>,
    pub context: Context,
    pub goal: Goal,
    pub first: u32,
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
}
pub(super) struct NormalizedInputs {
    pub proof: Option<usize>,
}
pub(super) struct Trial {
    pub context: Context,
    pub goal: Goal,
    pub instance: Instance,
    pub first: u32,
    pub end: u32,
    pub pending: Vec<usize>,
    pub cursor: usize,
    pub next: Vec<usize>,
    pub progress: bool,
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
        let mut context = Context::default();
        let goal = context
            .import_predicate(&facts.types, source, goal)
            .map_err(inference)?;
        Ok(Self {
            normalization: None,
            normalized_inputs: None,
            context,
            goal,
            first,
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
        let mut context = Context::default();
        let goal = context
            .import_predicate(&facts.types, &mut frame.context, &frame.goal)
            .map_err(inference)?;
        let first = frame.available_region().max(floor);
        let instance = instantiation::instantiate(facts, &mut context, owner, first)?;
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
            context,
            goal,
            instance,
            first,
            end,
            pending,
            cursor: 0,
            next: Vec::new(),
            progress: false,
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
            let ty = match trial.context.resolved(&mut facts.types, goal.subject()) {
                Ok(ty) => ty,
                Err(infer::Error::Unresolved) => return Ok(Some(Decision::Deferred)),
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
        let goal = match trial
            .context
            .resolved_predicate(&mut facts.types, &trial.goal)
        {
            Ok(goal) => goal,
            Err(infer::Error::Unresolved) => return Ok(Some(Decision::Deferred)),
            Err(error) => return Err(inference(error)),
        };
        let arguments = match trial.instance.arguments(facts, &mut trial.context) {
            Ok(arguments) => arguments,
            Err(infer::Error::Unresolved) => return Ok(Some(Decision::Deferred)),
            Err(error) => return Err(inference(error)),
        };
        let node = self.query.nodes.len();
        self.query.nodes.push(Node {
            goal,
            source: Some(Source::Implementation {
                owner: trial.instance.owner,
                arguments,
            }),
            children: std::mem::take(&mut trial.children),
            first: trial.first,
            end: trial.end,
            equalities: std::mem::take(&mut trial.context.region_equalities),
            outlives: std::mem::take(&mut trial.instance.outlives),
            type_outlives: requirements.type_outlives,
            sized: requirements.sized,
        });
        Ok(Some(Decision::Proven(node)))
    }
}
