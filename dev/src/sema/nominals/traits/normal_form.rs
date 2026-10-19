//! Canonical projections retain opened lifetimes and selection across header dependencies.
mod opening;
use super::{
    Assumptions, Availability, Declarations, Environment, Instantiation, Obligations, Predicate,
    Projected, State, selection,
};
use crate::{
    resolve::model::DefId,
    sema::traits::equalities,
    types::{
        infer::predicates::Predicate as Goal,
        parameters::predicates::Bound,
        projection::Application,
        substitute::{Request, Resolution},
    },
};
use std::sync::Arc;

pub(in crate::sema) enum Progress {
    Ready(Projected),
    Pending(DefId, Box<Query>),
}
pub(in crate::sema) struct Query {
    state: State,
    application: Application,
    requirement: Predicate,
    goal: Goal,
    assumptions: Option<Arc<Assumptions>>,
    pending: Option<Box<selection::Pending>>,
    source: Option<selection::Source>,
    closed: bool,
    frontier: u32,
    template: bool,
}

/// The enclosing normalizer retains the formation proof of every request input.
pub(in crate::sema) fn begin(
    facts: &mut Declarations,
    environment: Option<Arc<Environment>>,
    request: Request,
    first: u32,
    retain_sized: bool,
    availability: Availability,
) -> Result<Progress, &'static str> {
    if !facts
        .associated
        .families
        .contains_key(&DefId(request.application.item.0))
        && let Some((source, resolution)) =
            selection::known_normalized(facts, environment.as_deref(), request)
                .map_err(|_| "invalid or exhausted canonical projection premise")?
    {
        return Ok(Progress::Ready(Projected {
            source: Some(source),
            resolution: Some(resolution),
            proof: Obligations::new(first),
            sized: Vec::new(),
            family: request.application.arguments,
        }));
    }
    let requirement = Predicate {
        ty: request.application.subject,
        bound: Bound::Trait(request.application.trait_application),
    };
    let (mut state, requirement, arguments) = opening::prepare(
        facts,
        environment,
        (requirement, request.application.arguments),
        first,
        retain_sized,
        availability,
    )?;
    let goal = state.projection_goal(facts, requirement)?;
    let frontier = state.obligations.regions.end;
    Query {
        state,
        application: Application {
            arguments,
            ..request.application
        },
        requirement,
        goal,
        assumptions: None,
        pending: None,
        source: None,
        closed: false,
        frontier,
        template: false,
    }
    .advance(facts)
}

impl Query {
    pub fn allocation_start(&self) -> u32 {
        self.state.obligations.regions.start
    }

    /// Canonical declaration templates own a private namespace. No body may
    /// consume these IDs; restoration and fresh rebasing precede publication.
    pub fn resume_template(mut self, facts: &mut Declarations) -> Result<Progress, &'static str> {
        self.template = true;
        let first = self.allocation_end();
        self.resume(facts, first)
    }

    /// This includes a live child's IDs without claiming them in the parent proof.
    pub fn allocation_end(&self) -> u32 {
        self.pending
            .as_ref()
            .map_or(self.state.obligations.regions.end, |pending| {
                pending.allocation_end()
            })
            .max(self.frontier)
    }

    pub fn resume(
        mut self,
        facts: &mut Declarations,
        first: u32,
    ) -> Result<Progress, &'static str> {
        if self.pending.is_none() || first < self.allocation_end() {
            return Err("projection selection allocation frontier moved backwards");
        }
        self.frontier = first;
        self.advance(facts)
    }

    fn advance(mut self, facts: &mut Declarations) -> Result<Progress, &'static str> {
        loop {
            let selected = match self.select(facts)? {
                selection::Progress::Inferred { .. } => {
                    return Err("non-refining projection selection returned partial inputs");
                }
                selection::Progress::Pending(owner, pending) => {
                    self.pending = Some(pending);
                    return Ok(Progress::Pending(owner, Box::new(self)));
                }
                selection::Progress::Ready(selected) => self.state.accept_selection(selected)?,
            };
            // Failed trials may have reserved IDs before the wait. They carry
            // no constraints and cannot turn foreign allocations into evidence.
            if !self.template {
                self.state
                    .obligations
                    .resume_at(self.frontier.max(self.state.obligations.regions.end))?;
            }
            if self.closed {
                return if matches!(selected, Some(selection::Source::Implementation { .. })) {
                    self.source = selected;
                    self.finish(facts, None)
                } else {
                    self.finish(facts, Some(Resolution::Opaque))
                };
            }
            self.source = selected;
            if !matches!(
                selected,
                Some(
                    selection::Source::Assumption(_)
                        | selection::Source::ResultBound(_)
                        | selection::Source::QuantifiedAssumption { .. }
                        | selection::Source::QuantifiedResultBound { .. }
                )
            ) {
                return self.finish(facts, None);
            }
            match self.equality(facts)? {
                equalities::Selection::Proven(evidence) => {
                    let ty = evidence.append_to(&mut self.state.obligations)?;
                    return self.finish(facts, Some(Resolution::Type(ty)));
                }
                equalities::Selection::Ambiguous => {
                    return Err("ambiguous associated type equality assumptions");
                }
                equalities::Selection::Absent if self.requirement.is_concrete(&facts.types)? => {
                    self.closed = true;
                    self.goal = self.state.projection_goal(facts, self.requirement)?;
                }
                equalities::Selection::Absent => {
                    return self.finish(facts, Some(Resolution::Opaque));
                }
            }
        }
    }

    fn select(&mut self, facts: &mut Declarations) -> Result<selection::Progress, &'static str> {
        let state = &mut self.state;
        let selected = if let Some(pending) = self.pending.take() {
            pending.resume(facts, &mut state.context, &self.goal, self.frontier)
        } else if facts
            .associated
            .families
            .contains_key(&DefId(self.application.item.0))
        {
            let crate::types::parameters::predicates::Bound::Trait(trait_application) =
                self.requirement.bound
            else {
                return Err("missing family parent predicate");
            };
            let application = Application {
                subject: self.requirement.ty,
                trait_application,
                ..self.application
            };
            let caller = state.owned_caller(&mut self.assumptions, self.closed);
            selection::begin_projection(
                facts,
                &mut state.context,
                (&self.goal, application),
                caller,
                state.availability,
            )
        } else if facts
            .generics
            .implementations
            .as_ref()
            .is_none_or(|registry| registry.constant_headers_ready())
        {
            let empty = Assumptions::default();
            let caller = selection::Caller {
                assumptions: if self.closed {
                    &empty
                } else {
                    self.assumptions.as_deref().unwrap_or(&state.assumptions)
                },
                environment: if self.closed {
                    None
                } else {
                    state.environment.as_ref()
                },
                first_region: state.obligations.regions.end,
                retain_sized: state.retain_sized,
            };
            let select = if self.closed {
                selection::select_available
            } else {
                selection::select_normalized
            };
            select(
                facts,
                &mut state.context,
                &self.goal,
                caller,
                state.availability,
            )
            .map(selection::Progress::Ready)
        } else {
            let caller = state.owned_caller(&mut self.assumptions, self.closed);
            let begin = if self.closed {
                selection::begin
            } else {
                selection::begin_normalized
            };
            begin(
                facts,
                &mut state.context,
                &self.goal,
                caller,
                state.availability,
            )
        };
        selected.map_err(|_| "invalid or exhausted canonical trait selection")
    }

    fn equality(
        &mut self,
        facts: &mut Declarations,
    ) -> Result<equalities::Selection, &'static str> {
        let predicate = self.state.projection_goal(facts, self.requirement)?;
        let inferred = crate::types::infer::projections::Application {
            item: self.application.item,
            predicate,
            arguments: self
                .state
                .context
                .known_arguments(&facts.types, self.application.arguments)
                .map_err(|_| "invalid canonical family arguments")?,
        };
        equalities::for_projection(
            facts,
            &mut self.state.context,
            &inferred,
            self.state.environment.as_deref(),
            self.state.obligations.regions.end,
        )
        .map_err(|_| "invalid or exhausted associated equality selection")
    }

    fn finish(
        mut self,
        facts: &mut Declarations,
        mut resolution: Option<Resolution>,
    ) -> Result<Progress, &'static str> {
        let sized = std::mem::take(&mut self.state.sized);
        let proof = self.state.finish();
        if let Some(Resolution::Type(ty)) = &mut resolution {
            *ty = proof.restoration()?.apply(&mut facts.types, *ty)?;
        }
        Ok(Progress::Ready(Projected {
            source: self.source,
            resolution,
            proof,
            sized,
            family: self.application.arguments,
        }))
    }
}

impl State {
    fn owned_caller(
        &mut self,
        assumptions: &mut Option<Arc<Assumptions>>,
        closed: bool,
    ) -> selection::OwnedCaller {
        selection::OwnedCaller {
            assumptions: if closed {
                Arc::default()
            } else {
                assumptions
                    .get_or_insert_with(|| Arc::new(std::mem::take(&mut self.assumptions)))
                    .clone()
            },
            environment: if closed {
                None
            } else {
                self.environment.clone()
            },
            first_region: self.obligations.regions.end,
            retain_sized: self.retain_sized,
        }
    }

    fn projection_goal(
        &mut self,
        facts: &mut Declarations,
        requirement: Predicate,
    ) -> Result<Goal, &'static str> {
        let mut substitution = Instantiation::new(&mut self.context, &facts.types, &[])
            .map_err(|_| "invalid declaration trait instantiation")?;
        self.context
            .instantiate_predicate(&mut facts.types, requirement, &mut substitution)
            .map_err(|_| "invalid declaration trait goal")
    }
}
