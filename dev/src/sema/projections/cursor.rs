//! An owned semantic cursor retains the exact fold and selection across definition waits.
use super::{Declarations, DefId, Definition, Obligations, ScopeId, TypeId};
use crate::support::hash::HashMap;
use crate::{
    sema::{
        nominals::traits::{Projected, normal_form},
        traits::implementations::Availability,
    },
    types::{
        lifetime_proof::map_scopes,
        parameters::environment::Environment,
        substitute::{self, Bound, Normalizer, Request, Resolution, Step},
    },
};
use std::sync::Arc;

/// Concrete proofs without debts can outlive a single normalization's witnesses.
/// The owner must retain the same environment and implementation availability.
#[derive(Default)]
pub(in crate::sema) struct Completed(HashMap<u16, crate::sema::traits::selection::Reuse>);

#[derive(Debug, PartialEq, Eq)]
pub(in crate::sema) enum Progress {
    Complete { ty: TypeId, steps: usize },
    Definition(DefId),
    Dependency(crate::sema::traits::selection::Dependency),
}
struct Header {
    request: Request,
    query: Box<normal_form::Query>,
    frontier: Option<u32>,
}
struct Selected {
    request: Request,
    definition: Definition,
    restore: Option<Bound>,
}

/// Keep one proof, environment and provider for the cursor's lifetime. Before
/// yielding to other allocation work, its owner must reserve newly emitted proof
/// identities; on resumption the same proof resumes at the allocator frontier.
/// A definition wait never calls the provider or probes implementations again.
pub(in crate::sema) struct Cursor {
    environment: Option<Arc<Environment>>,
    ambient: ScopeId,
    depth: u16,
    availability: Availability,
    fold: Normalizer,
    scopes: Vec<ScopeId>,
    environments: HashMap<u16, Option<Arc<Environment>>>,
    reusable: Completed,
    step: Option<Step>,
    selected: Option<Box<Selected>>,
    header: Option<Header>,
    before: usize,
}
impl Cursor {
    pub fn new(
        environment: Option<Arc<Environment>>,
        proof: &Obligations,
        ambient: ScopeId,
        availability: Availability,
    ) -> Result<Self, &'static str> {
        Ok(Self {
            environment,
            ambient,
            depth: proof.scope_depth(ambient)?,
            availability,
            fold: Normalizer::default(),
            scopes: Vec::new(),
            environments: HashMap::default(),
            reusable: Completed::default(),
            step: None,
            selected: None,
            header: None,
            before: 0,
        })
    }

    pub fn start(&mut self, facts: &mut Declarations, ty: TypeId) -> Result<(), &'static str> {
        if self.step.is_some() || self.selected.is_some() || self.header.is_some() {
            return Err("normalization cursor has an unfinished query");
        }
        self.before = self.fold.steps();
        self.step = Some(self.fold.start(&mut facts.types, ty)?);
        Ok(())
    }

    pub fn exchange_completed(&mut self, completed: &mut Completed) {
        std::mem::swap(&mut self.reusable, completed);
    }

    pub fn allocation_end(&self, proof: &Obligations) -> u32 {
        self.header
            .as_ref()
            .map_or(proof.regions.end, |header| header.query.allocation_end())
    }

    pub fn resume_at(&mut self, proof: &mut Obligations, first: u32) -> Result<(), &'static str> {
        if let Some(header) = &mut self.header {
            if first < header.query.allocation_end() {
                return Err("projection cursor allocation frontier moved backwards");
            }
            header.frontier = Some(first);
            Ok(())
        } else {
            proof.resume_at(first)
        }
    }

    pub fn advance<F>(
        &mut self,
        facts: &mut Declarations,
        proof: &mut Obligations,
        local: &mut F,
    ) -> Result<Progress, &'static str>
    where
        F: FnMut(&mut Declarations, Request) -> Result<Option<TypeId>, &'static str>,
    {
        let mut step = if let Some(selected) = &mut self.selected {
            let Some(mut ty) = selected.definition.result(facts)? else {
                return Ok(Progress::Definition(selected.definition.id));
            };
            if let Some(restore) = &mut selected.restore {
                ty = restore.apply(&mut facts.types, ty)?;
            }
            let step =
                self.fold
                    .resume(&mut facts.types, selected.request, Resolution::Type(ty))?;
            self.selected = None;
            step
        } else if let Some(header) = &self.header {
            Step::Resolve(header.request)
        } else {
            self.step.take().ok_or("normalization cursor is inactive")?
        };
        loop {
            let request = match step {
                Step::Complete(ty) => {
                    return Ok(Progress::Complete {
                        ty,
                        steps: self.fold.steps().saturating_sub(self.before),
                    });
                }
                Step::Resolve(request) => request,
            };
            let progress = if let Some(header) = self.header.take() {
                let first = header
                    .frontier
                    .ok_or("projection cursor was not resumed after a header wait")?;
                header.query.resume(facts, first)?
            } else {
                if let Some(ty) = local(facts, request)? {
                    let resolution = if matches!(request.alias, substitute::Alias::TraitResult(..))
                        && facts.types.kind(ty) == Some(request.alias.kind())
                    {
                        Resolution::Opaque
                    } else {
                        Resolution::Type(ty)
                    };
                    step = self.fold.resume(&mut facts.types, request, resolution)?;
                    continue;
                }
                let environment = self.environment(facts, request.depth)?;
                let reuse = if facts.types.nominal_sizes_sealed()
                    && request.alias.associated().is_some_and(|application| {
                        facts.types.contains_projected_sizes(application.subject) == Some(true)
                    }) {
                    Some(self.reusable.0.entry(request.depth).or_default())
                } else {
                    None
                };
                normal_form::begin_reusing(
                    facts,
                    environment,
                    request,
                    proof.regions.end,
                    false,
                    self.availability,
                    reuse,
                )?
            };
            let projected = match progress {
                normal_form::Progress::Ready(projected) => projected,
                normal_form::Progress::Pending(owner, query) => {
                    self.header = Some(Header {
                        request,
                        query,
                        frontier: None,
                    });
                    return Ok(Progress::Dependency(owner));
                }
            };
            let resolution = self.selected_resolution(facts, request, &projected)?;
            let scope = map_scopes(
                &mut self.scopes,
                self.ambient,
                request.scope,
                |id| self.fold.binder(id).ok_or("missing normalization binder"),
                |parent, count| proof.scope(parent, count),
            )?;
            proof.append_streamed_at(projected.proof, scope)?;
            if let Some(resolution) = resolution {
                step = self.fold.resume(&mut facts.types, request, resolution)?;
            } else {
                return self
                    .selected
                    .as_ref()
                    .map(|selected| Progress::Definition(selected.definition.id))
                    .ok_or("missing retained projection selection");
            }
        }
    }

    fn selected_resolution(
        &mut self,
        facts: &mut Declarations,
        request: Request,
        projected: &Projected,
    ) -> Result<Option<Resolution>, &'static str> {
        let source = projected
            .source
            .ok_or("associated type requires an unambiguous validated implementation")?;
        if let Some(resolution) = projected.resolution {
            return Ok(Some(resolution));
        }
        let Some(definition) = Definition::alias(facts, request.alias, projected.family, source)?
        else {
            return Ok(Some(Resolution::Opaque));
        };
        let mut restore =
            if projected.proof.regions.is_empty() && projected.proof.equalities.is_empty() {
                None
            } else {
                Some(projected.proof.restoration()?)
            };
        if let Some(mut ty) = definition.result(facts)? {
            if let Some(restore) = &mut restore {
                ty = restore.apply(&mut facts.types, ty)?;
            }
            Ok(Some(Resolution::Type(ty)))
        } else {
            self.selected = Some(Box::new(Selected {
                request,
                definition,
                restore,
            }));
            Ok(None)
        }
    }

    fn environment(
        &mut self,
        facts: &mut Declarations,
        depth: u16,
    ) -> Result<Option<Arc<Environment>>, &'static str> {
        let local = self
            .depth
            .checked_add(depth)
            .ok_or("projection scope depth exhausted")?;
        if local == 0 {
            return Ok(self.environment.clone());
        }
        if let Some(existing) = self.environments.get(&local) {
            return Ok(existing.clone());
        }
        let shifted = self
            .environment
            .as_ref()
            .map(|env| {
                env.substitute_regions(&mut facts.types, &mut substitute::Bound::shift(local))
                    .map(Arc::new)
            })
            .transpose()?;
        self.environments.insert(local, shifted.clone());
        Ok(shifted)
    }
}
