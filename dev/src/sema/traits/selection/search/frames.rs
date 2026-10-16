//! Search frames keep alternatives isolated and delay builtin conditions until equations settle.
use super::{Context, Decision, Declarations, Goal, Node, Predicate, Search, Source, inference};
use crate::{
    resolve::model::DefId,
    sema::traits::selection::instantiation::{self, Instance},
    types::{infer, parameters::predicates::Bound},
};

pub(super) struct Frame {
    pub equality: Option<Box<super::equalities::Equality>>,
    pub context: Context,
    pub goal: Goal,
    pub first: u32,
    pub started: bool,
    pub key: Option<Predicate>,
    pub canonical: Option<Predicate>,
    pub cycles: usize,
    pub candidates: Vec<DefId>,
    pub cursor: usize,
    pub trial: Option<Trial>,
    pub selected: Option<usize>,
    pub deferred: bool,
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
            equality: None,
            context,
            goal,
            first,
            started: false,
            key: None,
            canonical: None,
            cycles: 0,
            candidates: Vec::new(),
            cursor: 0,
            trial: None,
            selected: None,
            deferred: false,
        })
    }
}
impl Trial {
    pub fn new(facts: &mut Declarations, frame: &mut Frame, owner: DefId) -> Result<Self, String> {
        let mut context = Context::default();
        let goal = context
            .import_predicate(&facts.types, &mut frame.context, &frame.goal)
            .map_err(inference)?;
        let instance = instantiation::instantiate(facts, &mut context, owner, frame.first)?;
        let end = frame.first
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
            first: frame.first,
            end,
            pending,
            cursor: 0,
            next: Vec::new(),
            progress: false,
            children: Vec::new(),
        })
    }
}
impl Search<'_> {
    pub(super) fn finish(
        &mut self,
        facts: &mut Declarations,
        trial: &mut Trial,
    ) -> Result<Decision, String> {
        let mut type_outlives = Vec::new();
        let mut sized = Vec::new();
        for goal in &trial.instance.goals {
            let Some(bound) = goal.builtin_bound() else {
                continue;
            };
            let ty = match trial.context.resolved(&mut facts.types, goal.subject()) {
                Ok(ty) => ty,
                Err(infer::Error::Unresolved) => return Ok(Decision::Deferred),
                Err(error) => return Err(inference(error)),
            };
            if let Bound::Outlives(region) = bound {
                type_outlives.push((ty, region));
            } else if !self.builtins.prove(facts, Predicate { ty, bound })? {
                return Ok(Decision::Absent);
            } else if bound == Bound::Sized && self.caller.retain_sized {
                sized.push(ty);
            }
        }
        let goal = match trial
            .context
            .resolved_predicate(&mut facts.types, &trial.goal)
        {
            Ok(goal) => goal,
            Err(infer::Error::Unresolved) => return Ok(Decision::Deferred),
            Err(error) => return Err(inference(error)),
        };
        let arguments = match trial.instance.arguments(facts, &mut trial.context) {
            Ok(arguments) => arguments,
            Err(infer::Error::Unresolved) => return Ok(Decision::Deferred),
            Err(error) => return Err(inference(error)),
        };
        let node = self.nodes.len();
        self.nodes.push(Node {
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
            type_outlives,
            sized,
        });
        Ok(Decision::Proven(node))
    }
}
