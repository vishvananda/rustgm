//! Completed body inputs normalize with validated implementations in their actual scopes.
use super::{
    model::Declarations,
    nominals::traits::State,
    traits::{implementations::Availability, selection::Source},
};
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind, Namespace},
    },
    types::{
        ItemId, TypeId,
        lifetime_proof::{Obligations, ScopeId, map_scopes},
        parameters::{
            environment::Environment,
            predicates::{Bound, Predicate},
        },
        substitute::{self, Normalizer, Resolution, Step},
    },
};
use std::{collections::HashMap, sync::Arc};

pub(super) fn definition(
    report: &Report,
    facts: &Declarations,
    owner: DefId,
    item: ItemId,
) -> Result<DefId, &'static str> {
    let name = report.resolution.definitions[item.0 as usize].name;
    let scope = report.resolution.definitions[owner.0 as usize].scope;
    report.resolution.scopes[scope.0 as usize]
        .bindings
        .get(&(name, Namespace::Type))
        .map(|b| b.target)
        .filter(|id| {
            report.resolution.definitions[id.0 as usize].kind == DefKind::TypeAlias
                && facts.associated.owner(*id) == facts.associated.owner(owner)
        })
        .ok_or("selected implementation is missing the associated type")
}

pub(super) fn selected_result(
    report: &Report,
    facts: &mut Declarations,
    item: ItemId,
    source: Source,
    selected: &Obligations,
) -> Result<TypeId, &'static str> {
    let Source::Implementation { owner, arguments } = source else {
        return Err("associated type requires an unambiguous validated implementation");
    };
    let definition = definition(report, facts, owner, item)?;
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
    report: &Report,
    facts: &mut Declarations,
    environment: Option<&Arc<Environment>>,
    proof: &mut Obligations,
    ambient: ScopeId,
    ty: TypeId,
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
        let predicate = Predicate {
            ty: request.application.subject,
            bound: Bound::Trait(request.application.trait_application),
        };
        let (source, selected, _) = State::projection(
            facts,
            environment,
            predicate,
            proof.regions.end,
            false,
            Availability::Validated,
        )?;
        let Some(source) = source else {
            return Err("associated type requires an unambiguous validated implementation");
        };
        let ty = selected_result(report, facts, request.application.item, source, &selected)?;
        let scope = map_scopes(
            &mut scopes,
            ambient,
            request.scope,
            |id| fold.binder(id).ok_or("missing normalization binder"),
            |parent, count| proof.scope(parent, count),
        )?;
        proof.append_at(selected, scope)?;
        step = fold.resume(&mut facts.types, request, Resolution::Type(ty))?;
    }
}
