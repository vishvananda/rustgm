//! An owned semantic cursor retains the exact fold and selection across definition waits.
#[cfg(test)]
#[path = "../../../tests/invariants/nested_condition_universes.rs"]
mod scoped_body_tests;
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
pub(in crate::sema) struct Completed(HashMap<u16, Retained>);
#[derive(Default)]
struct Retained {
    proofs: crate::sema::traits::selection::Reuse,
    sizes: Vec<crate::sema::nominals::sizes::Learned>,
}

#[derive(Debug, PartialEq, Eq)]
pub(in crate::sema) enum Progress {
    Complete {
        ty: TypeId,
        steps: usize,
        nested: u32,
    },
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
    callable_depth: u16,
    availability: Availability,
    fold: Normalizer,
    scopes: Vec<ScopeId>,
    environments: HashMap<u16, Option<Arc<Environment>>>,
    reusable: Completed,
    step: Option<Step>,
    selected: Option<Box<Selected>>,
    header: Option<Header>,
    before: usize,
    nested: u32,
    placeholders: Option<crate::types::parameters::shape::Placeholders>,
    /// The level of the trait proof that owns this cursor; each query it
    /// starts is one unit below it.
    nesting: crate::sema::traits::selection::Nesting,
    /// Whether this cursor has started a query.
    used: bool,
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
            callable_depth: proof.scope_plan().callable_depth(ambient)?,
            availability,
            fold: Normalizer::default(),
            scopes: Vec::new(),
            environments: HashMap::default(),
            reusable: Completed::default(),
            step: None,
            selected: None,
            header: None,
            before: 0,
            nested: 0,
            placeholders: None,
            nesting: crate::sema::traits::selection::Nesting::ROOT,
            used: false,
        })
    }

    /// The premises of the placeholder trait proof that owns this cursor; its
    /// selections decide a rigid parameter's sizing only from them. They are
    /// installed once, before the cursor starts a query or holds reused proofs.
    pub(in crate::sema) fn assume_placeholders(
        &mut self,
        placeholders: Option<crate::types::parameters::shape::Placeholders>,
    ) -> Result<(), &'static str> {
        if placeholders.is_some() && self.environment.is_some() {
            return Err("placeholder premises require a premise-less normalization");
        }
        if self.placeholders.is_some() || self.used || !self.reusable.0.is_empty() {
            return Err(crate::sema::nominals::PLACEHOLDER_REINSTALL);
        }
        self.placeholders = placeholders;
        Ok(())
    }

    /// The level of the trait proof that owns this cursor.
    pub(in crate::sema) const fn nest_at(
        &mut self,
        nesting: crate::sema::traits::selection::Nesting,
    ) {
        self.nesting = nesting;
    }

    pub fn start(&mut self, facts: &mut Declarations, ty: TypeId) -> Result<(), &'static str> {
        if self.step.is_some() || self.selected.is_some() || self.header.is_some() {
            return Err("normalization cursor has an unfinished query");
        }
        self.used = true;
        if let Some(retained) = self.reusable.0.get_mut(&0) {
            retained.sizes.clear();
            retained.proofs.set_physical(None);
        }
        self.nested = 0;
        self.before = self.fold.steps();
        self.step = Some(self.fold.start(&mut facts.types, ty)?);
        Ok(())
    }

    pub fn start_sizing(
        &mut self,
        facts: &mut Declarations,
        ty: TypeId,
        physical: Option<Box<crate::sema::nominals::sizes::Physical>>,
    ) -> Result<(), &'static str> {
        self.start(facts, ty)?;
        if physical.is_some() {
            self.reusable
                .0
                .entry(0)
                .or_default()
                .proofs
                .set_physical(physical);
        }
        Ok(())
    }
    pub fn take_sizes(&mut self) -> Vec<crate::sema::nominals::sizes::Learned> {
        self.reusable
            .0
            .get_mut(&0)
            .map_or_else(Vec::new, |retained| std::mem::take(&mut retained.sizes))
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
                        nested: self.nested,
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
                    Some(&mut self.reusable.0.entry(request.depth).or_default().proofs)
                } else {
                    None
                };
                normal_form::begin_reusing(
                    facts,
                    environment,
                    (request, self.callable_depth),
                    proof.regions.end,
                    normal_form::Sizing {
                        retain_sized: false,
                        placeholders: self.placeholders.as_ref(),
                        nesting: self.nesting.nested(),
                    },
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
            let resolution = self.retain_projected(facts, proof, request, projected)?;
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

    /// Pay a completed nested query and retain its actual scoped proof before
    /// continuing the fold or waiting for the selected definition.
    fn retain_projected(
        &mut self,
        facts: &mut Declarations,
        proof: &mut Obligations,
        request: Request,
        projected: Projected,
    ) -> Result<Option<Resolution>, &'static str> {
        self.nested = self
            .nested
            .saturating_add(u32::try_from(projected.work).unwrap_or(u32::MAX));
        if self
            .fold
            .steps()
            .saturating_sub(self.before)
            .saturating_add(self.nested as usize)
            > 1_000_000
        {
            return Err("projection normalization work budget exceeded");
        }
        let resolution = self.selected_resolution(facts, request, &projected)?;
        let scope = map_scopes(
            &mut self.scopes,
            self.ambient,
            request.scope,
            |id| self.fold.binder(id).ok_or("missing normalization binder"),
            |parent, count| proof.callable_scope(parent, count),
        )?;
        proof.append_streamed_at(projected.proof, scope)?;
        if self.depth == 0
            && request.depth == 0
            && !projected.provisional_size
            && let Some(learned) = projected.learned
        {
            self.reusable.0.entry(0).or_default().sizes.push(*learned);
        }
        Ok(resolution)
    }

    fn selected_resolution(
        &mut self,
        facts: &mut Declarations,
        request: Request,
        projected: &Projected,
    ) -> Result<Option<Resolution>, &'static str> {
        let source = projected
            .source
            .clone()
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
