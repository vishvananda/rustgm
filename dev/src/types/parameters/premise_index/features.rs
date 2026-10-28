//! Feature membership shares the complete immutable snapshot across index views.
use super::{Arc, ClauseIndex, HashSet};
use crate::{
    support::retained::Growth,
    types::{Store, infer, parameters::features::FeatureId},
};
#[derive(Clone)]
pub(in crate::types::parameters) struct Set {
    pub(super) arena: crate::types::arena::Identity,
    pub(super) values: HashSet<FeatureId>,
}
impl ClauseIndex {
    /// Concrete callers retain non-type membership without symbolic trait evidence.
    pub(crate) fn feature_only_charged(
        &self,
        types: &Store,
        pay: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Option<Arc<Self>>, &'static str> {
        pay(1, 0)?;
        let Some(features) = &self.features else {
            return Ok(None);
        };
        if !types.has_identity(&features.arena) {
            return Err("feature index belongs to a different type arena");
        }
        pay(1, size_of::<Self>() + 2 * size_of::<usize>())?;
        Ok(Some(Arc::new(Self {
            features: Some(Arc::clone(features)),
            ..Self::default()
        })))
    }
    pub(crate) fn contains_feature(
        &self,
        types: &Store,
        id: FeatureId,
        pay: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<bool, &'static str> {
        pay(1, 0)?;
        types
            .feature_name(id)
            .ok_or("foreign feature index query")?;
        if let Some(features) = &self.features {
            if !types.has_identity(&features.arena) {
                return Err("feature index belongs to a different type arena");
            }
            return Ok(features.values.contains(&id));
        }
        Ok(false)
    }
    pub(super) fn insert_feature<const CHARGED: bool, E: From<infer::Error>>(
        &mut self,
        types: &Store,
        id: FeatureId,
        pay: &impl Fn(usize, usize) -> Result<(), E>,
    ) -> Result<(), E> {
        types
            .feature_name(id)
            .ok_or(infer::Error::InvalidIdentity)?;
        if self
            .features
            .as_ref()
            .is_some_and(|features| features.values.contains(&id))
        {
            return Ok(());
        }
        let (len, capacity) = self.features.as_ref().map_or((0, 0), |features| {
            (features.values.len(), features.values.capacity())
        });
        if len >= 1_000_000 {
            return Err(infer::Error::Budget.into());
        }
        if CHARGED {
            let mut growth = Growth::map::<FeatureId, ()>(len, capacity, 1);
            if self.features.is_none() {
                growth.bytes = growth
                    .bytes
                    .saturating_add(size_of::<Set>() + 2 * size_of::<usize>());
            } else if self
                .features
                .as_ref()
                .is_some_and(|features| Arc::strong_count(features) != 1)
            {
                growth.include(Growth::map::<FeatureId, ()>(0, 0, len));
                growth.bytes = growth
                    .bytes
                    .saturating_add(size_of::<Set>() + 2 * size_of::<usize>());
            }
            pay(growth.work.saturating_add(1), growth.bytes)?;
        }
        let arena = types
            .existing_arena_identity()
            .ok_or(infer::Error::InvalidIdentity)?;
        if self
            .features
            .as_ref()
            .is_some_and(|features| !types.has_identity(&features.arena))
        {
            return Err(infer::Error::InvalidIdentity.into());
        }
        let features = self.features.get_or_insert_with(|| {
            Arc::new(Set {
                arena: arena.clone(),
                values: HashSet::default(),
            })
        });
        Arc::make_mut(features).values.insert(id);
        Ok(())
    }
}
