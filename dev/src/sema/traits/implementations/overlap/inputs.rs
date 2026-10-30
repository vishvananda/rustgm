//! Existential type equations retain each header's separate lifetime premises.
use super::{Declarations, DefId, Predicate};
use crate::types::{
    RegionId, TypeId,
    infer::{self, Context, Instantiation, TermId, arguments::Argument},
    lifetime_proof::Obligations,
    outlives::Queries,
    parameters::{ParameterKind, clauses::Clause, predicates::Bound},
    relations::Constraints,
    substitute,
};

#[derive(Default)]
pub(super) struct Inputs {
    pub sized: Vec<TermId>,
    traits: Vec<infer::predicates::Predicate>,
    universals: u32,
    regions: Vec<(RegionId, RegionId)>,
    outlives: Vec<(TermId, RegionId)>,
    formed: Vec<TermId>,
}
impl Inputs {
    pub fn instantiate(
        &mut self,
        facts: &mut Declarations,
        context: &mut Context,
        (predicate, owner): (Predicate, DefId),
    ) -> Result<infer::predicates::Predicate, String> {
        self.reserve_traits(facts, context, owner)?;
        let ids = facts.generics.parameter_ids(&facts.types, owner)?;
        let mut bindings = Vec::with_capacity(ids.len());
        for id in ids {
            let argument = context
                .parameter_variable(&facts.types, id)
                .map_err(|e| e.to_string())?;
            if let Argument::Type(term) = argument
                && matches!(
                    facts.types.parameter(id).map(|p| p.kind),
                    Some(ParameterKind::Type { sized: true })
                )
            {
                self.sized.push(term);
            }
            bindings.push((id, argument));
        }
        let mut substitution =
            Instantiation::new(context, &facts.types, &bindings).map_err(|e| e.to_string())?;
        let mut lifetimes = self.lifetimes(facts, owner)?;
        for requirement in facts.generics.predicates_in_scope(owner) {
            if let Clause::UnstableFeature(id) = requirement.value {
                context.account_work(1).map_err(|e| e.to_string())?;
                facts
                    .types
                    .feature_name(id)
                    .ok_or("invalid overlap feature condition")?;
                // An unavailable feature never proves two implementations disjoint.
                continue;
            }
            let Clause::Predicate(predicate) = requirement.value else {
                continue;
            };
            if predicate.bound.trait_parts().is_some() {
                let predicate = lifetimes.predicate(&mut facts.types, predicate)?;
                self.traits.push(context.instantiate_predicate(&mut facts.types, predicate, &mut substitution).map_err(|e| e.to_string())?);
                continue;
            }
            if !matches!(
                predicate.bound,
                Bound::Sized | Bound::Copy | Bound::Clone | Bound::Outlives(_)
            ) {
                continue;
            }
            let predicate = lifetimes.predicate(&mut facts.types, predicate)?;
            let term = context
                .instantiate_with(&mut facts.types, predicate.ty, &mut substitution)
                .map_err(|e| e.to_string())?;
            if let Bound::Outlives(region) = predicate.bound {
                self.outlives.push((term, region));
            } else {
                self.sized.push(term);
            }
        }
        let predicate = lifetimes.predicate(&mut facts.types, predicate)?;
        context
            .account_work(lifetimes.steps)
            .map_err(|e| e.to_string())?;
        let goal = context
            .instantiate_predicate(&mut facts.types, predicate, &mut substitution)
            .map_err(|e| e.to_string())?;
        self.formed
            .extend(goal.arguments().filter_map(|arg| match arg {
                Argument::Type(ty) => Some(ty),
                _ => None,
            }));
        Ok(goal)
    }

    fn reserve_traits(
        &mut self,
        facts: &mut Declarations,
        context: &mut Context,
        owner: DefId,
    ) -> Result<(), String> {
        let mut count = 0_usize;
        for requirement in facts.generics.predicates_in_scope(owner) {
            context.account_work(1).map_err(|e| e.to_string())?;
            if matches!(requirement.value, Clause::Predicate(value) if value.bound.trait_parts().is_some()) {
                count = count.saturating_add(1);
            }
        }
        if count != 0 {
            let growth = crate::support::retained::Growth::vector::<infer::predicates::Predicate>(self.traits.len(), self.traits.capacity(), count);
            // Ordinary PA20 crates have no source-header ledger. Do not create
            // one after declaration completion merely to account temporary work.
            context.account_work(growth.work.saturating_add(count).saturating_add(growth.bytes)).map_err(|e| e.to_string())?;
            if facts.must_use_contracts.as_deref().and_then(|owner| owner.headers.as_ref()).is_some() {
                crate::sema::header_attributes::charge_attribute_retention(facts, growth.work.saturating_add(count), growth.bytes).map_err(|e| e.to_string())?;
            }
            self.traits.try_reserve(count).map_err(|_| "coherence trait conditions allocation failed")?;
        }
        Ok(())
    }

    fn lifetimes(
        &mut self,
        facts: &Declarations,
        owner: DefId,
    ) -> Result<substitute::Bound, String> {
        let schema = facts
            .associated
            .schema(owner)
            .ok_or("missing coherence header schema")?;
        let mut names = Vec::with_capacity(schema.captures as usize);
        for _ in 0..schema.captures {
            let slot =
                u16::try_from(self.universals).map_err(|_| "too many coherence lifetimes")?;
            names.push(RegionId::bound(0, slot)?);
            self.universals += 1;
        }
        let mut mapping = substitute::Bound::new(names);
        for &(a, b) in &schema.bounds {
            self.regions
                .push((mapping.region(a, 0)?, mapping.region(b, 0)?));
        }
        Ok(mapping)
    }

    /// Only local, fully closed applications can establish compatible-world
    /// absence. Open/foreign/projected conditions remain possible.
    pub fn has_absent_local_trait(
        &self,
        facts: &mut Declarations,
        context: &mut Context,
    ) -> Result<bool, String> {
        for goal in &self.traits {
            context.account_work(1).map_err(|e| e.to_string())?;
            let predicate = match context.resolved_predicate(&mut facts.types, goal) {
                Ok(value) => value,
                Err(infer::Error::Unresolved | infer::Error::Normalization) => continue,
                Err(error) => return Err(error.to_string()),
            };
            if !predicate.is_concrete(&facts.types)? || predicate.contains_opaques(&facts.types)? {
                continue;
            }
            let (application, _) = predicate.bound.trait_parts().ok_or("coherence trait condition lost its application")?;
            let item = facts.types.trait_application(application).ok_or("coherence trait application is missing")?.item;
            let local = facts.associated.owner(DefId(item.0)).and_then(|id| facts.associated.get(id))
                .is_some_and(|owner| owner.source.is_some() && matches!(owner.kind,
                    crate::sema::associated::OwnerKind::Trait { definition, .. } if definition == DefId(item.0)));
            if local && !crate::sema::traits::selection::may_overlap(facts, context, goal)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn prove(
        &self,
        facts: &mut Declarations,
        context: &mut Context,
        proof: &Obligations,
    ) -> Result<(), String> {
        if proof.is_empty() {
            return Ok(());
        }
        let mut formed = Vec::new();
        for &term in &self.formed {
            if let Some(ty) = resolved(facts, context, term)? {
                formed.push(ty);
            }
        }
        let mut outlives = Vec::new();
        for &(term, region) in &self.outlives {
            if let Some(ty) = resolved(facts, context, term)? {
                outlives.push((ty, region));
            }
        }
        let mut constraints = Constraints::new(
            &facts.types,
            &facts.lifetimes.variances,
            self.universals,
            1_000_000,
        )
        .map_err(|e| e.to_string())?;
        let mut queries = Queries::new(
            &facts.types,
            &facts.lifetimes.nominal_requirements,
            &facts.lifetimes.callables,
        );
        for &(a, b) in &self.regions {
            constraints.assume(a, b).map_err(|e| e.to_string())?;
        }
        for ty in formed {
            constraints.assume_well_formed(&mut queries, ty)?;
        }
        for (ty, region) in outlives {
            constraints.assume_type_outlives(&mut queries, ty, region)?;
        }
        proof.require(&mut constraints, &mut queries)?;
        constraints.solve().map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn resolved(
    facts: &mut Declarations,
    context: &mut Context,
    term: TermId,
) -> Result<Option<TypeId>, String> {
    context.account_work(1).map_err(|e| e.to_string())?;
    match context.resolved(&mut facts.types, term) {
        Ok(ty) => Ok(Some(ty)),
        Err(infer::Error::Unresolved | infer::Error::Normalization) => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}
