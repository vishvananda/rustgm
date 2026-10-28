//! Source-free non-type proof keeps caller premises and global policy separate.
#[cfg(test)]
#[path = "../../../tests/invariants/feature_predicates.rs"]
mod tests;
use crate::{
    sema::model::Declarations,
    support::{hash::HashSet, retained::Growth},
    types::{
        Store,
        arena::Identity,
        parameters::{environment::Environment, features::FeatureId},
    },
};

pub(super) struct Policy {
    arena: Identity,
    enabled: HashSet<FeatureId>,
}
impl Policy {
    fn new(
        types: &mut Store,
        enabled: &[FeatureId],
        pay: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Self, &'static str> {
        pay(1, size_of::<Self>() + Identity::allocation_bytes())?;
        let mut result = Self {
            arena: types.arena_identity(),
            enabled: HashSet::default(),
        };
        for &id in enabled {
            pay(1, 0)?;
            types
                .feature_name(id)
                .ok_or("global feature belongs to another store")?;
            let growth = Growth::map::<FeatureId, ()>(
                result.enabled.len(),
                result.enabled.capacity(),
                usize::from(!result.enabled.contains(&id)),
            );
            pay(growth.work, growth.bytes)?;
            result.enabled.insert(id);
        }
        Ok(result)
    }
    fn prove(
        &self,
        types: &Store,
        id: FeatureId,
        caller: Option<&Environment>,
        pay: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<bool, &'static str> {
        pay(1, 0)?;
        if !types.has_identity(&self.arena) || types.feature_name(id).is_none() {
            return Err("feature query changed its actual type arena");
        }
        if caller
            .map(|caller| caller.assumes_feature(types, id, pay))
            .transpose()?
            .unwrap_or(false)
        {
            return Ok(true);
        }
        pay(1, 0)?;
        Ok(self.enabled.contains(&id))
    }
}
impl Declarations {
    /// The authenticated producer supplies enabling once; empty means caller-only.
    pub(crate) fn publish_feature_policy(
        &mut self,
        enabled: &[FeatureId],
        pay: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        pay(1, 0)?;
        if self
            .generics
            .traits
            .as_ref()
            .is_some_and(|contracts| contracts.feature_policy.is_some())
        {
            return Err("immutable feature policy was already published");
        }
        if self.generics.traits.is_none() {
            pay(1, size_of::<super::Contracts>())?;
        }
        let policy = Policy::new(&mut self.types, enabled, pay)?;
        self.generics
            .traits
            .get_or_insert_with(Default::default)
            .feature_policy = Some(Box::new(policy));
        Ok(())
    }
    pub(crate) fn prove_feature(
        &self,
        id: FeatureId,
        caller: Option<&Environment>,
        pay: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<bool, &'static str> {
        pay(1, 0)?;
        self.generics
            .traits
            .as_ref()
            .and_then(|contracts| contracts.feature_policy.as_deref())
            .ok_or("non-type feature proof requires its authenticated immutable policy")?
            .prove(&self.types, id, caller, pay)
    }
}
