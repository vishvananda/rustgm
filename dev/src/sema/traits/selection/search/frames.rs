//! Search frames keep alternatives isolated and delay builtin conditions until equations settle.
use super::{
    Action, Context, Decision, Declarations, Goal, Node, Predicate, Search, Source, inference,
};
use crate::{
    resolve::model::DefId,
    sema::traits::selection::instantiation::{self, Instance},
    types::{infer, lifetime_proof::ScopeId, parameters::predicates::Bound, shape_key::ShapeKey},
};

#[derive(Clone, Copy)]
pub(super) enum Chosen {
    Positive(usize),
    Negative(usize),
}
impl Chosen {
    pub(super) fn node(self) -> usize {
        match self {
            Self::Positive(node) | Self::Negative(node) => node,
        }
    }
    pub(super) fn decision(self) -> Decision {
        match self {
            Self::Positive(node) => Decision::Proven(node),
            Self::Negative(node) => Decision::Refuted(node),
        }
    }
}

pub(super) struct Frame {
    pub language: Option<Box<super::mechanical::Pending>>,
    pub equality_hint: Option<Box<crate::sema::traits::callables::Matched>>,
    pub storage: Option<Box<super::sized_obligations::Pending>>,
    pub autos: Option<Box<super::auto_obligations::Pending>>,
    pub explicit_auto: bool,
    pub formation: Option<Box<super::member_conditions::Formation>>,
    pub normalization: Option<Box<super::normalization::Normalization>>,
    /// A suspended normalizer has proved these inputs in this search's scope.
    /// Its evidence must accompany any result derived without repeating that work.
    pub normalized_inputs: Option<NormalizedInputs>,
    pub inputs: infer::refinement::Refinement,
    pub goal: Goal,
    pub first: u32,
    pub scope: ScopeId,
    pub auto_segment: usize,
    pub started: bool,
    pub key: Option<ShapeKey>,
    pub canonical: Option<Predicate>,
    pub cycles: usize,
    pub candidates: Vec<DefId>,
    pub pending_headers: Option<std::vec::IntoIter<DefId>>,
    pub cursor: usize,
    pub trial: Option<Trial>,
    pub selected: Option<Chosen>,
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
            language: None,
            equality_hint: None,
            storage: None,
            autos: None,
            explicit_auto: false,
            formation: None,
            normalization: None,
            normalized_inputs: None,
            inputs,
            goal,
            first,
            scope: ScopeId::ROOT,
            auto_segment: 0,
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
        let instance =
            instantiation::instantiate(facts, &mut inputs.context, owner, first, goal.constness())?;
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
    /// Resume one isolated candidate's pending proof frontier before considering alternatives.
    pub(super) fn advance_trial(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Action, String> {
        let trial = frame
            .trial
            .as_mut()
            .ok_or("missing active implementation candidate")?;
        if trial.cursor == trial.pending.len() {
            trial.pending.clear();
            trial.cursor = 0;
        }
        trial.poll_condition_changes(facts, None)?;
        if let Some(&index) = trial.pending.get(trial.cursor) {
            trial.wakeups.start(index);
            return Ok(Action::Descend);
        }
        let result = if trial.wakeups.waiting() {
            Decision::Deferred
        } else {
            let Some(result) = self.finish(facts, trial)? else {
                return Ok(Action::Descend);
            };
            result
        };
        self.charge(&mut trial.inputs.context)?;
        let retained = frame.trial.take();
        if !matches!(result, Decision::Proven(_)) {
            self.query.invalidate_auto(frame);
        }
        match result {
            Decision::Absent => {}
            Decision::Deferred
                if self.query.refining
                    && frame.inference != Inference::Disabled
                    && frame.scope == self.query.input_scope()
                    && !frame.normalizing() =>
            {
                if frame.inference == Inference::Committed {
                    frame.trial = retained;
                    return Ok(Action::Yield);
                }
                if frame.partial.is_some() {
                    frame.deferred = true;
                } else {
                    frame.partial = retained.map(Box::new);
                }
            }
            Decision::Deferred => frame.deferred = true,
            Decision::Proven(node) | Decision::Refuted(node) => {
                let chosen = if matches!(result, Decision::Proven(_)) {
                    Chosen::Positive(node)
                } else {
                    Chosen::Negative(node)
                };
                frame.selected = Some(if let Some(previous) = frame.selected {
                    let Some(preferred) = self.preferred_candidate(
                        facts,
                        previous,
                        chosen,
                        frame.canonical.is_some(),
                    )?
                    else {
                        return Ok(Action::Complete(Decision::Deferred));
                    };
                    preferred
                } else {
                    chosen
                });
            }
        }
        Ok(Action::Continue)
    }
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
                || matches!(bound, Bound::Auto(_))
                || bound == Bound::Sized
                    && (facts.types.contains_projected_sizes(ty) == Some(true)
                        || !facts.types.nominal_sizes_sealed()
                            && facts.types.contains_nominals(ty) == Some(true))
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
        match self.conditions(facts, trial, arguments)? {
            Some(false) => return Ok(Some(Decision::Absent)),
            None => {
                trial.requirements = Some(Box::new(requirements));
                return Ok(None);
            }
            Some(true) => {}
        }
        if facts.generics.implementations.as_ref()
            .ok_or("reservation trial lost its actual registry")?
            .polarity(&facts.types, &facts.associated, trial.instance.owner)?
            == crate::sema::traits::implementations::Polarity::Reservation
        {
            if self.query.purpose != crate::sema::traits::implementations::Purpose::Coherence {
                return Err("reservation cannot complete ordinary trait selection".into());
            }
            // Conditions have been checked; this possibility still cannot
            // publish a selected implementation, substitutions or proof node.
            return Ok(Some(Decision::Deferred));
        }
        let owned_end = trial.first
            + u32::try_from(trial.instance.regions.len())
                .map_err(|_| "impl region count overflow")?;
        let node = self.query.push(
            Node {
                debt_free: false,
                cycle_dependent: false,
                provisional_size: false,
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
        let negative =
            super::super::refutation::implementation_negative(facts, trial.instance.owner)?;
        Ok(Some(if negative {
            Decision::Refuted(node)
        } else {
            Decision::Proven(node)
        }))
    }
}

impl Search<'_> {
    /// Each alternative already owns its complete isolated proof. A sealed
    /// graph may order those answers only for a fully resolved requesting goal;
    /// inference alternatives and every non-graph ambiguity keep deferring.
    fn preferred_candidate(
        &self,
        facts: &Declarations,
        previous: Chosen,
        next: Chosen,
        known: bool,
    ) -> Result<Option<Chosen>, String> {
        let (Chosen::Positive(a), Chosen::Positive(b)) = (previous, next) else {
            return Ok(None);
        };
        if !known {
            return Ok(None);
        }
        let owner = |index: usize| -> Result<Option<DefId>, &'static str> {
            Ok(
                match self
                    .query
                    .nodes
                    .get(index)
                    .ok_or("candidate proof disappeared")?
                    .source
                {
                    Some(Source::Implementation { owner, .. }) => Some(owner),
                    _ => None,
                },
            )
        };
        let (Some(a), Some(b)) = (owner(a)?, owner(b)?) else {
            return Ok(None);
        };
        Ok(crate::sema::specialization::prefer(facts, a, b)?
            .map(|winner| if winner == a { previous } else { next }))
    }
}
