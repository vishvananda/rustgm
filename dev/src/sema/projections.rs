//! Projection queries retain implementation authority and their actual proof scopes.
use super::{
    model::Declarations,
    nominals::traits::State,
    traits::{implementations::Availability, selection::Source},
};
use crate::{
    resolve::model::DefId,
    types::{
        ItemId, TypeId,
        lifetime_proof::{Obligations, ScopeId, map_scopes},
        parameters::environment::Environment,
        substitute::{self, Normalizer, Request, Resolution, Step},
    },
};
use std::{collections::HashMap, sync::Arc};

pub(super) fn definition(
    facts: &Declarations,
    owner: DefId,
    item: ItemId,
) -> Result<DefId, &'static str> {
    facts
        .associated
        .type_definition(owner, DefId(item.0))
        .ok_or("selected implementation is missing the associated type")
}

pub(super) fn selected_result(
    facts: &mut Declarations,
    item: ItemId,
    source: Source,
    selected: &Obligations,
) -> Result<TypeId, &'static str> {
    let ty = implementation_result(facts, item, source)?;
    if selected.regions.is_empty() && selected.equalities.is_empty() {
        Ok(ty)
    } else {
        selected.restoration()?.apply(&mut facts.types, ty)
    }
}

/// Keep private result lifetimes in the selected proof's allocation interval.
pub(super) fn implementation_result(
    facts: &mut Declarations,
    item: ItemId,
    source: Source,
) -> Result<TypeId, &'static str> {
    let Source::Implementation { owner, arguments } = source else {
        return Err("associated type requires an unambiguous validated implementation");
    };
    let definition = definition(facts, owner, item)?;
    let template = facts.definitions[definition.0 as usize]
        .ok_or("missing completed associated type definition")?;
    facts
        .generics
        .substitute(&mut facts.types, owner, arguments, template)
}

pub(super) fn normalize(
    facts: &mut Declarations,
    environment: Option<&Arc<Environment>>,
    proof: &mut Obligations,
    ambient: ScopeId,
    ty: TypeId,
    availability: Availability,
) -> Result<(TypeId, usize), &'static str> {
    Session::new(
        environment.cloned(),
        proof,
        ambient,
        availability,
        |_, _| Ok(None),
    )?
    .normalize(facts, ty)
}

/// Completed roots share work only within one fixed environment, proof scope,
/// implementation availability and local definition provider. Evidence stays in
/// the borrowed proof for the whole session, including nested callable scopes.
pub(crate) struct Session<'proof, F> {
    environment: Option<Arc<Environment>>,
    proof: &'proof mut Obligations,
    ambient: ScopeId,
    depth: u16,
    availability: Availability,
    local: F,
    fold: Normalizer,
    scopes: Vec<ScopeId>,
    environments: HashMap<u16, Option<Arc<Environment>>>,
}
impl<'proof, F> Session<'proof, F>
where
    F: FnMut(&mut Declarations, Request) -> Result<Option<TypeId>, &'static str>,
{
    /// The provider must prove a result for the complete application in the
    /// request's binder context; it cannot change authority between roots.
    pub(super) fn new(
        environment: Option<Arc<Environment>>,
        proof: &'proof mut Obligations,
        ambient: ScopeId,
        availability: Availability,
        local: F,
    ) -> Result<Self, &'static str> {
        let depth = proof.scope_depth(ambient)?;
        Ok(Self {
            environment,
            proof,
            ambient,
            depth,
            availability,
            local,
            fold: Normalizer::default(),
            scopes: Vec::new(),
            environments: HashMap::new(),
        })
    }
    pub(crate) fn normalize(
        &mut self,
        facts: &mut Declarations,
        ty: TypeId,
    ) -> Result<(TypeId, usize), &'static str> {
        let before = self.fold.steps();
        let mut step = self.fold.start(&mut facts.types, ty)?;
        loop {
            let request = match step {
                Step::Complete(ty) => return Ok((ty, self.fold.steps().saturating_sub(before))),
                Step::Resolve(request) => request,
            };
            if let Some(ty) = (self.local)(facts, request)? {
                step = self
                    .fold
                    .resume(&mut facts.types, request, Resolution::Type(ty))?;
                continue;
            }
            let local = self
                .depth
                .checked_add(request.depth)
                .ok_or("projection scope depth exhausted")?;
            let environment = if local == 0 {
                self.environment.clone()
            } else if let Some(existing) = self.environments.get(&local) {
                existing.clone()
            } else {
                let shifted = self
                    .environment
                    .as_ref()
                    .map(|env| {
                        env.substitute_regions(
                            &mut facts.types,
                            &mut substitute::Bound::shift(local),
                        )
                        .map(Arc::new)
                    })
                    .transpose()?;
                self.environments.insert(local, shifted.clone());
                shifted
            };
            let projected = State::projection(
                facts,
                environment,
                request.application,
                self.proof.regions.end,
                false,
                self.availability,
            )?;
            let Some(source) = projected.source else {
                return Err("associated type requires an unambiguous validated implementation");
            };
            let resolution = if let Some(resolution) = projected.resolution {
                resolution
            } else {
                Resolution::Type(selected_result(
                    facts,
                    request.application.item,
                    source,
                    &projected.proof,
                )?)
            };
            let scope = map_scopes(
                &mut self.scopes,
                self.ambient,
                request.scope,
                |id| self.fold.binder(id).ok_or("missing normalization binder"),
                |parent, count| self.proof.scope(parent, count),
            )?;
            self.proof.append_at(projected.proof, scope)?;
            step = self.fold.resume(&mut facts.types, request, resolution)?;
        }
    }
}

/// Instantiated types have no symbolic caller trait permissions. Selection uses
/// validated implementations and retains all lifetime debts in the instance.
pub(crate) type Concrete<'proof> =
    Session<'proof, fn(&mut Declarations, Request) -> Result<Option<TypeId>, &'static str>>;

pub(crate) fn concrete(
    proof: &mut Obligations,
    scope: ScopeId,
) -> Result<Concrete<'_>, &'static str> {
    Session::new(None, proof, scope, Availability::Validated, |_, _| Ok(None))
}
