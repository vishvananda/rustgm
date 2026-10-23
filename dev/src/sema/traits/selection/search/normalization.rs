//! Normalization requests descend on the existing proof stack and retain their evidence.
use super::{Action, Decision, Declarations, Frame, Goal, Search, Source, inference};
use crate::sema::{projections::Definition, traits::equalities};
use crate::types::{
    TypeId,
    infer::{Instantiation, projections::Application},
    lifetime_proof::ScopeId,
    substitute::{Alias, Normalizer, Resolution, Step},
};

pub(super) enum Purpose {
    Equality(super::equalities::Equality),
    Inputs(Box<super::inputs::Inputs>),
}
pub(super) struct Normalization {
    pub fold: Normalizer,
    pub step: Step,
    pub purpose: Purpose,
    opened: Option<Opened>,
    pub end: u32,
    pub proof: Option<usize>,
    pub stopped: Option<Decision>,
    pub scope: ScopeId,
    pub scoped: Option<Box<super::callable_queries::Scoped>>,
}
struct Opened {
    goal: Goal,
    application: Alias,
}
struct Completion {
    opened: Opened,
    node: usize,
    equations_start: usize,
}
impl Normalization {
    pub fn pending(&self) -> bool {
        !matches!(&self.purpose, Purpose::Inputs(inputs) if inputs.ready)
    }
    pub fn new(
        facts: &mut Declarations,
        frame: &mut Frame,
        root: TypeId,
        purpose: Purpose,
    ) -> Result<Self, String> {
        let mut fold = Normalizer::default();
        let step = fold.start(&mut facts.types, root)?;
        frame
            .inputs
            .context
            .account_work(fold.steps())
            .map_err(inference)?;
        Ok(Self {
            fold,
            step,
            purpose,
            opened: None,
            end: frame.available_region(),
            proof: None,
            stopped: None,
            scope: frame.scope,
            scoped: None,
        })
    }
}
impl Frame {
    pub(super) fn child(
        &mut self,
        facts: &mut Declarations,
        floor: u32,
        suspended: &mut [Option<Frame>],
    ) -> Result<Self, String> {
        if let Some(normalization) = &self.normalization
            && normalization.pending()
        {
            let opened = normalization
                .opened
                .as_ref()
                .ok_or("missing normalization application")?;
            let mut child = Self::new(
                facts,
                &mut self.inputs.context,
                &opened.goal,
                normalization.end.max(floor),
            )?;
            child.scope = normalization.scope;
            child.normalized_inputs = Some(super::frames::NormalizedInputs {
                proof: normalization.proof,
            });
            if let Some(application) = opened.application.associated() {
                child.prepare_formation(facts, application)?;
            }
            return Ok(child);
        }
        let trial = self.trial.as_mut().ok_or("missing descending candidate")?;
        let index = trial.pending[trial.cursor];
        if let Some(slot) = trial.wakeups.take_child(index) {
            let mut child = suspended
                .get_mut(slot)
                .and_then(Option::take)
                .ok_or("missing retained partial child")?;
            child.refresh(facts, &mut trial.inputs.context)?;
            return Ok(child);
        }
        let mut child = Self::new(
            facts,
            &mut trial.inputs.context,
            &trial.instance.goals[index],
            trial.end.max(floor),
        )?;
        child.scope = self.scope;
        child.inference = if self.inference == super::frames::Inference::Disabled {
            super::frames::Inference::Disabled
        } else {
            super::frames::Inference::Selecting
        };
        Ok(child)
    }
    pub(super) fn available_region(&self) -> u32 {
        let start = self
            .equality_hint
            .as_ref()
            .map_or(self.first, |hint| self.first.max(hint.regions.end));
        let first = self
            .normalization
            .as_ref()
            .map_or(start, |normalization| start.max(normalization.end));
        let first = self
            .storage
            .as_ref()
            .map_or(first, |storage| first.max(storage.end));
        self.autos
            .as_ref()
            .map_or(first, |autos| first.max(autos.end))
    }
    pub(super) fn normalizing(&self) -> bool {
        self.normalization
            .as_ref()
            .is_some_and(|normalization| normalization.pending())
    }
}
impl Search<'_> {
    pub(super) fn advance_normalization(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Action, String> {
        let normalization = frame
            .normalization
            .as_mut()
            .ok_or("missing normalization query")?;
        if let Some(decision) = normalization.stopped {
            return Ok(Action::Complete(decision));
        }
        let value =
            match normalization.step {
                Step::Complete(value) => value,
                Step::Resolve(request) => {
                    let mut application = request.alias;
                    if request.depth != 0 || normalization.scoped.is_some() {
                        let scoped = normalization.scoped.get_or_insert_with(|| {
                            Box::new(super::callable_queries::Scoped::new(
                                frame.scope,
                                frame.first,
                            ))
                        });
                        normalization.scope = scoped.request(
                            &mut self.query,
                            &mut frame.inputs.context,
                            &normalization.fold,
                            request.scope,
                        )?;
                        if normalization.scope != scoped.active.scope {
                            let universes = self
                                .query
                                .scopes
                                .as_mut()
                                .ok_or("missing normalization universes")?;
                            let mut current = normalization.scope;
                            while current != scoped.active.scope {
                                let (next, steps, parent) = universes
                                    .open_normalization_application(facts, current, application)?;
                                frame
                                    .inputs
                                    .context
                                    .account_work(steps)
                                    .map_err(inference)?;
                                application = next;
                                current = parent;
                            }
                        }
                    } else {
                        normalization.scope = frame.scope;
                    }
                    let mut substitution =
                        Instantiation::new(&mut frame.inputs.context, &facts.types, &[])
                            .map_err(inference)?;
                    let predicate =
                        crate::sema::opaque::overrides::requirement(facts, application)?;
                    normalization.opened = Some(Opened {
                        application,
                        goal: frame
                            .inputs
                            .context
                            .instantiate_predicate(&mut facts.types, predicate, &mut substitution)
                            .map_err(inference)?,
                    });
                    return Ok(Action::Descend);
                }
            };
        match normalization.purpose {
            Purpose::Equality(_) => self.advance_equality(facts, frame, value),
            Purpose::Inputs(_) => self.advance_inputs(facts, frame, value),
        }
    }
    pub(super) fn accept_normalization(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        decision: Decision,
    ) -> Result<(), String> {
        let equality = frame
            .normalization
            .as_mut()
            .ok_or("missing parent normalization")?;
        let Decision::Proven(mut node) = decision else {
            equality.stopped = Some(decision);
            return Ok(());
        };
        let Step::Resolve(request) = equality.step else {
            return Err("missing pending equality normalization".into());
        };
        let opened = equality
            .opened
            .take()
            .ok_or("missing equality application")?;
        let equations_start = frame.inputs.context.region_equalities.len();
        frame
            .inputs
            .context
            .equate_trait_predicate(&mut facts.types, &opened.goal, self.query.nodes[node].goal)
            .map_err(inference)?;
        let (resolution, selected) = self.resolve_normalization(
            facts,
            &mut frame.inputs.context,
            equality,
            Completion {
                opened,
                node,
                equations_start,
            },
        )?;
        let Some(mut resolution) = resolution else {
            equality.stopped = Some(Decision::Deferred);
            return Ok(());
        };
        node = selected;
        if equality.scope != frame.scope {
            let equations = frame
                .inputs
                .context
                .region_equalities
                .split_off(equations_start);
            node = self
                .query
                .normalization_equations(node, equality.scope, equations);
        }
        if let Some(scoped) = &mut equality.scoped {
            resolution = scoped.selected(
                &self.query,
                facts,
                &mut frame.inputs.context,
                equality.scope,
                node,
                resolution,
            )?;
        }
        if self.query.nodes[node].first != self.query.nodes[node].end {
            equality.end = self.query.nodes[node].end;
        }
        equality.proof = Some(self.inherit_inputs(node, equality.proof));
        frame.inputs.context.account_work(2).map_err(inference)?;
        let before = equality.fold.steps();
        equality.step = equality
            .fold
            .resume(&mut facts.types, request, resolution)?;
        frame
            .inputs
            .context
            .account_work(equality.fold.steps().saturating_sub(before))
            .map_err(inference)
    }
    fn resolve_normalization(
        &mut self,
        facts: &mut Declarations,
        context: &mut super::Context,
        equality: &Normalization,
        completed: Completion,
    ) -> Result<(Option<Resolution>, usize), String> {
        let Completion {
            opened,
            mut node,
            equations_start,
        } = completed;
        let source = self.query.nodes[node]
            .source
            .ok_or("missing equality premise evidence")?;
        let resolution = match source {
            Source::Implementation { .. }
            | Source::Callable { .. }
            | Source::LanguageOperation { .. } => {
                if let Some(application) = opened.application.associated() {
                    Resolution::Type(crate::sema::projections::implementation_result(
                        facts,
                        application,
                        source,
                    )?)
                } else if let Some(definition) = Definition::alias(
                    facts,
                    opened.application,
                    opened.application.arguments(),
                    source,
                )? {
                    Resolution::Type(
                        definition
                            .result(facts)?
                            .ok_or("missing completed associated result")?,
                    )
                } else {
                    Resolution::Opaque
                }
            }
            Source::Assumption(_)
            | Source::ResultBound(_)
            | Source::QuantifiedAssumption { .. }
            | Source::QuantifiedResultBound { .. } => {
                let Some(application) = opened.application.associated() else {
                    return Ok((Some(Resolution::Opaque), node));
                };
                let application = Application {
                    item: application.item,
                    predicate: opened.goal,
                    arguments: context
                        .known_arguments(&facts.types, opened.application.arguments())
                        .map_err(inference)?,
                };
                match equalities::for_projection(
                    facts,
                    context,
                    &application,
                    self.caller.environment.map(AsRef::as_ref),
                    self.query.nodes[node]
                        .end
                        .max(equality.end)
                        .max(self.query.allocations.floor()),
                )? {
                    equalities::Selection::Proven(evidence) => {
                        let ty = evidence.ty;
                        let mut equations = context.region_equalities.split_off(equations_start);
                        node = self.query.equality_use(
                            node,
                            equality.scope,
                            evidence,
                            &mut equations,
                        )?;
                        context.region_equalities.extend(equations);
                        Resolution::Type(ty)
                    }
                    equalities::Selection::Absent => Resolution::Opaque,
                    equalities::Selection::Ambiguous => {
                        return Ok((None, node));
                    }
                }
            }
            Source::Equality => return Err("equality evidence cannot prove a trait premise".into()),
            Source::Builtin => return Err("builtin evidence cannot prove a trait premise".into()),
        };
        Ok((Some(resolution), node))
    }
}

impl super::Query {
    /// Equality instantiation belongs to the requesting scope, after its parent
    /// proof. Keep it in the selected graph so discarded alternatives grant no IDs.
    fn equality_use(
        &mut self,
        parent: usize,
        scope: crate::types::lifetime_proof::ScopeId,
        evidence: equalities::Evidence,
        equations: &mut Vec<crate::types::outlives::Pair>,
    ) -> Result<usize, &'static str> {
        let regions = evidence.premise.regions;
        if regions.is_empty() {
            return Ok(parent);
        }
        self.reserve(scope, regions.clone())?;
        self.allocations.record(regions.end);
        Ok(self.push(
            super::Node {
                debt_free: false,
                cycle_dependent: false,
                provisional_size: false,
                goal: evidence.premise.predicate,
                source: Some(Source::Equality),
                children: vec![parent],
                first: self.nodes[parent].first,
                end: regions.end,
                equalities: std::mem::take(equations),
                outlives: Vec::new(),
                type_outlives: Vec::new(),
                sized: Vec::new(),
            },
            scope,
            regions,
        ))
    }
}
