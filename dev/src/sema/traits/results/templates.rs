//! Close declaration rules once, then select candidates before mapping caller inputs.
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    types::{
        ItemId,
        parameters::predicates::{Bound, Predicate},
    },
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[derive(Default)]
pub(super) struct Index {
    candidates: HashMap<ItemId, Vec<Predicate>>,
    equalities: HashMap<ItemId, Vec<Predicate>>,
    pub(super) outlives: Vec<crate::types::RegionId>,
}
impl Index {
    pub(super) fn candidates(&self, item: ItemId) -> Option<&[Predicate]> {
        self.candidates.get(&item).map(Vec::as_slice)
    }
    pub(super) fn equalities(&self, item: ItemId) -> Option<&[Predicate]> {
        self.equalities.get(&item).map(Vec::as_slice)
    }
}

pub(super) fn for_result(
    facts: &mut Declarations,
    item: DefId,
) -> Result<(Arc<Index>, usize), String> {
    if let Some(index) = facts
        .generics
        .traits
        .as_ref()
        .and_then(|contracts| contracts.result_rules.templates.get(&item))
    {
        return Ok((Arc::clone(index), 1));
    }
    let contract = facts
        .associated
        .type_contract(item)
        .ok_or("missing associated result contract")?;
    let template = contract.projection;
    let mut pending: Vec<_> = contract
        .bounds
        .iter()
        .copied()
        .filter(|predicate| {
            (predicate.ty == template
                && matches!(predicate.bound, Bound::Trait(_) | Bound::Outlives(_)))
                || matches!(predicate.bound, Bound::ProjectionEqual(_))
        })
        .collect();
    let mut steps = contract.bounds.len();
    let mut seen = HashSet::new();
    let mut index = Index::default();
    while let Some(predicate) = pending.pop() {
        steps = steps.saturating_add(1);
        if steps.saturating_add(pending.len()) > 1_000_000 {
            return Err("associated result rule work budget exceeded".into());
        }
        if !seen.insert(predicate) {
            continue;
        }
        if let Bound::ProjectionEqual(_) = predicate.bound {
            if let Some(crate::types::Kind::Projection(application)) =
                facts.types.kind(predicate.ty)
                && application.subject == template
            {
                index
                    .equalities
                    .entry(application.item)
                    .or_default()
                    .push(predicate);
            }
            continue;
        }
        if predicate.ty != template {
            continue;
        }
        if let Bound::Outlives(region) = predicate.bound {
            index.outlives.push(region);
            continue;
        }
        let Bound::Trait(application) = predicate.bound else {
            continue;
        };
        let application = facts
            .types
            .trait_application(application)
            .ok_or("missing result rule application")?;
        index
            .candidates
            .entry(application.item)
            .or_default()
            .push(predicate);
        // Ordinary Self supertraits are finite. Associated result closure is
        // demanded by a later proof; recursive result bounds must not expand here.
        let requirements = facts
            .generics
            .traits
            .as_mut()
            .ok_or("missing result trait requirements")?
            .requirements(&mut facts.types, predicate)?;
        for clause in &requirements.implied {
            pending.push(clause.ordinary()?);
        }
    }
    let index = Arc::new(index);
    let cache = &mut facts
        .generics
        .traits
        .as_mut()
        .ok_or("missing result rule contracts")?
        .result_rules;
    if cache.enabled {
        cache.account(steps)?;
        cache.templates.insert(item, Arc::clone(&index));
    }
    Ok((index, steps))
}
