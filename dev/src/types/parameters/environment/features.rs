//! Non-type caller premises retain ordering without changing dense environments.
use super::Environment;
use crate::{
    support::{hash::HashSet, retained::Allocation},
    types::{Store, parameters::features::FeatureId},
};
#[derive(Clone, Default)]
pub(super) struct Premises {
    pub arena: Option<crate::types::arena::Identity>,
    pub entries: Vec<(usize, FeatureId)>,
    pub seen: HashSet<FeatureId>,
}
impl Premises {
    pub fn allocation(&self) -> Allocation {
        let mut allocation = Allocation::default();
        allocation.boxed(self);
        allocation.vector(&self.entries);
        allocation.set(&self.seen);
        allocation
    }
}
impl Environment {
    /// Add actual declared feature premises, paying cold storage before publication.
    /// # Errors
    /// Rejects foreign IDs, immutable query scopes and exhausted payments.
    pub fn with_feature_assumptions_charged(
        mut self,
        types: &Store,
        ids: &[FeatureId],
        pay: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Self, &'static str> {
        use crate::support::retained::Growth;
        use std::sync::Arc;
        pay(1, 0)?;
        if self.parameters.extended() || self.has_trait_opening() {
            return Err("feature premises require a mutable declaration scope");
        }
        for &id in ids {
            pay(1, 0)?;
            types
                .feature_name(id)
                .ok_or("foreign feature declaration premise")?;
            match &self.traits {
                None => pay(
                    1,
                    size_of::<super::premises::TraitAssumptions>() + 2 * size_of::<usize>(),
                )?,
                Some(traits) if Arc::strong_count(traits) != 1 => traits.prepay_clone(pay)?,
                Some(_) => {}
            }
            let features = self
                .traits
                .as_ref()
                .and_then(|traits| traits.features.as_ref());
            let additional =
                usize::from(features.is_none_or(|features| !features.seen.contains(&id)));
            let (len, capacity, seen, slots) = features.map_or((0, 0, 0, 0), |features| {
                (
                    features.entries.len(),
                    features.entries.capacity(),
                    features.seen.len(),
                    features.seen.capacity(),
                )
            });
            let mut growth = Growth::vector::<(usize, FeatureId)>(len, capacity, additional);
            growth.include(Growth::map::<FeatureId, ()>(seen, slots, additional));
            if features.is_none() {
                growth.bytes = growth.bytes.saturating_add(size_of::<Premises>());
            }
            pay(growth.work.saturating_add(1), growth.bytes)?;
            Arc::make_mut(self.traits.get_or_insert_with(Default::default))
                .insert(types, super::Clause::UnstableFeature(id))?;
        }
        Ok(self)
    }
    pub(crate) fn assumes_feature(
        &self,
        types: &Store,
        id: FeatureId,
        pay: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<bool, &'static str> {
        pay(1, 0)?;
        types
            .feature_name(id)
            .ok_or("foreign feature assumption query")?;
        let Some(traits) = &self.traits else {
            return Ok(false);
        };
        let traits = traits
            .mapped
            .as_ref()
            .map_or(traits.as_ref(), |mapped| mapped.source.as_ref());
        if let Some(features) = &traits.features
            && !features
                .arena
                .as_ref()
                .is_some_and(|arena| types.has_identity(arena))
        {
            return Err("feature caller premises belong to a different type arena");
        }
        Ok(traits
            .features
            .as_ref()
            .is_some_and(|features| features.seen.contains(&id)))
    }
}
