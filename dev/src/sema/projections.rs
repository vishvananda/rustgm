//! Completed body inputs normalize with validated implementations in their actual scopes.
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
        substitute::{self, Normalizer, Resolution, Step},
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
    let Source::Implementation { owner, arguments } = source else {
        return Err("associated type requires an unambiguous validated implementation");
    };
    let definition = definition(facts, owner, item)?;
    let template = facts.definitions[definition.0 as usize]
        .ok_or("missing completed associated type definition")?;
    let ty = facts
        .generics
        .substitute(&mut facts.types, owner, arguments, template)?;
    if selected.regions.is_empty() && selected.equalities.is_empty() {
        Ok(ty)
    } else {
        selected.restoration()?.apply(&mut facts.types, ty)
    }
}

pub(super) fn normalize(
    facts: &mut Declarations,
    environment: Option<&Arc<Environment>>,
    proof: &mut Obligations,
    ambient: ScopeId,
    ty: TypeId,
    availability: Availability,
) -> Result<(TypeId, usize), &'static str> {
    let depth = proof.scope_depth(ambient)?;
    let mut fold = Normalizer::default();
    let mut step = fold.start(&mut facts.types, ty)?;
    let mut scopes = Vec::new();
    let mut environments: HashMap<u16, Option<Arc<Environment>>> = HashMap::new();
    loop {
        let request = match step {
            Step::Complete(ty) => return Ok((ty, fold.steps())),
            Step::Resolve(request) => request,
        };
        let local = depth
            .checked_add(request.depth)
            .ok_or("projection scope depth exhausted")?;
        let environment = if local == 0 {
            environment.cloned()
        } else if let Some(existing) = environments.get(&local) {
            existing.clone()
        } else {
            let shifted = environment
                .as_ref()
                .map(|env| {
                    env.substitute_regions(&mut facts.types, &mut substitute::Bound::shift(local))
                        .map(Arc::new)
                })
                .transpose()?;
            environments.insert(local, shifted.clone());
            shifted
        };
        let projected = State::projection(
            facts,
            environment,
            request.application,
            proof.regions.end,
            false,
            availability,
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
            &mut scopes,
            ambient,
            request.scope,
            |id| fold.binder(id).ok_or("missing normalization binder"),
            |parent, count| proof.scope(parent, count),
        )?;
        proof.append_at(projected.proof, scope)?;
        step = fold.resume(&mut facts.types, request, resolution)?;
    }
}
