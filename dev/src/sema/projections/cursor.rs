//! An owned semantic cursor retains the exact fold and selection across definition waits.
use super::{Declarations, DefId, Definition, Obligations, ScopeId, TypeId};
use crate::{
    sema::{nominals::traits::State, traits::implementations::Availability},
    types::{
        lifetime_proof::map_scopes,
        parameters::environment::Environment,
        substitute::{self, Bound, Normalizer, Request, Resolution, Step},
    },
};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, PartialEq, Eq)]
pub(in crate::sema) enum Progress {
    Complete { ty: TypeId, steps: usize },
    Definition(DefId),
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
    step: Option<Step>,
    selected: Option<Box<Selected>>,
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
            environments: HashMap::new(),
            step: None,
            selected: None,
            before: 0,
        })
    }

    pub fn start(&mut self, facts: &mut Declarations, ty: TypeId) -> Result<(), &'static str> {
        if self.step.is_some() || self.selected.is_some() {
            return Err("normalization cursor has an unfinished query");
        }
        self.before = self.fold.steps();
        self.step = Some(self.fold.start(&mut facts.types, ty)?);
        Ok(())
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
            if let Some(ty) = local(facts, request)? {
                step = self
                    .fold
                    .resume(&mut facts.types, request, Resolution::Type(ty))?;
                continue;
            }
            let environment = self.environment(facts, request.depth)?;
            let projected = State::projection(
                facts,
                environment,
                request,
                proof.regions.end,
                false,
                self.availability,
            )?;
            let source = projected
                .source
                .ok_or("associated type requires an unambiguous validated implementation")?;
            let resolution = if let Some(resolution) = projected.resolution {
                Some(resolution)
            } else {
                let definition = Definition::select(facts, request.application.item, source)?;
                if let Some(mut ty) = definition.result(facts)? {
                    if !projected.proof.regions.is_empty() || !projected.proof.equalities.is_empty()
                    {
                        ty = projected.proof.restoration()?.apply(&mut facts.types, ty)?;
                    }
                    Some(Resolution::Type(ty))
                } else {
                    let restore = if projected.proof.regions.is_empty()
                        && projected.proof.equalities.is_empty()
                    {
                        None
                    } else {
                        Some(projected.proof.restoration()?)
                    };
                    self.selected = Some(Box::new(Selected {
                        request,
                        definition,
                        restore,
                    }));
                    None
                }
            };
            let scope = map_scopes(
                &mut self.scopes,
                self.ambient,
                request.scope,
                |id| self.fold.binder(id).ok_or("missing normalization binder"),
                |parent, count| proof.scope(parent, count),
            )?;
            proof.append_at(projected.proof, scope)?;
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
