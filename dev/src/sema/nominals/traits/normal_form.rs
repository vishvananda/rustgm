//! Canonical projections retain opened lifetimes and selection across header dependencies.
mod application_anchors;
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
        projection::Application,
        substitute::{Alias, Request, Resolution},
    },
};
use std::sync::Arc;

pub(in crate::sema) enum Progress {
    Ready(Projected),
    Pending(selection::Dependency, Box<Query>),
}
pub(in crate::sema) struct Query {
    state: State,
    alias: Alias,
    requested_arguments: crate::types::GenericArgs,
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
    begin_reusing(
        facts,
        environment,
        request,
        first,
        retain_sized,
        availability,
        None,
    )
}

pub(in crate::sema) fn begin_reusing(
    facts: &mut Declarations,
    environment: Option<Arc<Environment>>,
    request: Request,
    first: u32,
    retain_sized: bool,
    availability: Availability,
    reuse: Option<&mut selection::Reuse>,
) -> Result<Progress, &'static str> {
    if request.alias.associated().is_some_and(|application| {
        !facts
            .associated
            .families
            .contains_key(&DefId(application.item.0))
    }) && let Some((source, resolution)) =
        selection::known_normalized(facts, environment.as_deref(), request)
            .map_err(|_| "invalid or exhausted canonical projection premise")?
    {
        return Ok(Progress::Ready(Projected {
            provisional_size: false,
            source: Some(source),
            resolution: Some(resolution),
            proof: Obligations::new(first),
            sized: Vec::new(),
            family: request.alias.arguments(),
        }));
    }
    let requirement = crate::sema::opaque::overrides::requirement(facts, request.alias)?;
    let reuse = if environment
        .as_ref()
        .is_none_or(|env| !env.requires_region_mapping())
        && !requirement.has_free_regions(&facts.types)?
        && !facts
            .types
            .arguments_have_free_regions(request.alias.arguments())?
    {
        reuse
    } else {
        None
    };
    let (mut state, requirement, arguments) = opening::prepare(
        facts,
        environment,
        (requirement, request.alias.arguments()),
        first,
        retain_sized,
        availability,
    )?;
    let goal = state.projection_goal(facts, requirement)?;
    let frontier = state.obligations.regions.end;
    Query {
        state,
        alias: request.alias.with_arguments(arguments),
        requested_arguments: request.alias.arguments(),
        requirement,
        goal,
        assumptions: None,
        pending: None,
        source: None,
        closed: false,
        frontier,
        template: false,
    }
    .advance(facts, reuse)
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
        self.advance(facts, None)
    }

    fn advance(
        mut self,
        facts: &mut Declarations,
        mut reuse: Option<&mut selection::Reuse>,
    ) -> Result<Progress, &'static str> {
        loop {
            let selected = match self.select(facts, reuse.as_deref_mut())? {
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
            if let Some(source @ selection::Source::Callable { .. }) = selected {
                self.source = Some(source);
                let definition = crate::sema::projections::Definition::alias(
                    facts,
                    self.alias,
                    self.alias.arguments(),
                    source,
                )?
                .ok_or("callable evidence has no associated output")?;
                let result = definition.result(facts)?.ok_or("missing callable output")?;
                return self.finish(facts, Some(Resolution::Type(result)));
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
            if matches!(self.alias, Alias::TraitResult(..)) {
                if self.requirement.is_concrete(&facts.types)? {
                    self.closed = true;
                    self.goal = self.state.projection_goal(facts, self.requirement)?;
                    continue;
                }
                return self.finish(facts, Some(Resolution::Opaque));
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

    fn select(
        &mut self,
        facts: &mut Declarations,
        reuse: Option<&mut selection::Reuse>,
    ) -> Result<selection::Progress, &'static str> {
        let state = &mut self.state;
        let selected = if let Some(pending) = self.pending.take() {
            pending.resume(facts, &mut state.context, &self.goal, self.frontier)
        } else if let Some(application) = self.alias.associated().filter(|application| {
            facts
                .associated
                .families
                .contains_key(&DefId(application.item.0))
        }) {
            let crate::types::parameters::predicates::Bound::Trait(trait_application) =
                self.requirement.bound
            else {
                return Err("missing family parent predicate");
            };
            let application = Application {
                subject: self.requirement.ty,
                trait_application,
                ..application
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
            && !facts.types.nominal_sizes_pending()
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
            if !self.closed
                && let Some(reuse) = reuse
            {
                selection::select_reusing(
                    facts,
                    &mut state.context,
                    &self.goal,
                    caller,
                    (state.availability, true),
                    reuse,
                )
            } else {
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
            }
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
        let application = self
            .alias
            .associated()
            .ok_or("trait result has no written equality")?;
        let predicate = self.state.projection_goal(facts, self.requirement)?;
        let inferred = crate::types::infer::projections::Application {
            item: application.item,
            predicate,
            arguments: self
                .state
                .context
                .known_arguments(&facts.types, self.alias.arguments())
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
        let provisional_size = self.state.provisional_size;
        let proof = self.state.finish();
        if let Some(Resolution::Type(ty)) = &mut resolution {
            *ty = proof.restoration()?.apply(&mut facts.types, *ty)?;
        } else if resolution == Some(Resolution::Opaque)
            && let Some(ty) = application_anchors::restore(
                &mut facts.types,
                &proof,
                self.alias.with_arguments(self.requested_arguments),
                &mut facts.body_work.canonical_steps,
            )?
        {
            resolution = Some(Resolution::Type(ty));
        }
        Ok(Progress::Ready(Projected {
            provisional_size,
            source: self.source,
            resolution,
            proof,
            sized,
            family: self.alias.arguments(),
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
