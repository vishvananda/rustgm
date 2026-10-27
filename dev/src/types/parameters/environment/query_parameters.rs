//! One immutable query scope shares the complete declared parameter table.
use super::{Abilities, Environment, HashMap, ParameterId};
use crate::{
    support::retained::{self, Growth},
    types::{Store, arena::Identity, parameters::ParameterKind},
};
use std::sync::Arc;

#[derive(Clone, Default)]
pub(super) struct Parameters {
    pub(super) own: HashMap<ParameterId, Abilities>,
    prefix: Option<Arc<Self>>,
}
impl Parameters {
    pub(super) fn declared(own: HashMap<ParameterId, Abilities>) -> Self {
        Self { own, prefix: None }
    }
    pub(super) fn extended(&self) -> bool {
        self.prefix.is_some()
    }
    pub(super) fn get(&self, id: &ParameterId) -> Option<&Abilities> {
        self.own
            .get(id)
            .or_else(|| self.prefix.as_ref().and_then(|prefix| prefix.own.get(id)))
    }
    pub(super) fn contains_key(&self, id: &ParameterId) -> bool {
        self.get(id).is_some()
    }
    pub(super) fn get_mut(&mut self, id: &ParameterId) -> Option<&mut Abilities> {
        if self.prefix.is_some() {
            return None;
        }
        self.own.get_mut(id)
    }
    #[cfg(test)]
    pub(super) fn keys(&self) -> impl Iterator<Item = &ParameterId> {
        self.own
            .keys()
            .chain(self.prefix.iter().flat_map(|prefix| prefix.own.keys()))
    }
    pub(super) fn allocation(&self) -> retained::Allocation {
        let mut result = retained::Allocation::default();
        result.boxed(self);
        result.map(&self.own);
        result
    }
}
impl Environment {
    /// A compatibility query admits only the actual unsized source formal. The
    /// caller authenticates its declaration's PointeeSized bound independently.
    pub(crate) fn with_rigid_query_parameter(
        self: &Arc<Self>,
        types: &Store,
        arena: &Identity,
        id: ParameterId,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Arc<Self>, &'static str> {
        charge(1, 0)?;
        if !types.has_identity(arena) || self.parameters.extended() {
            return Err(
                "rigid query scope requires its actual arena and unextended declared prefix",
            );
        }
        if self.parameters.contains_key(&id)
            || !matches!(
                types.parameter(id).map(|p| p.kind),
                Some(ParameterKind::Type { sized: false })
            )
        {
            return Err("rigid query target is duplicate or lacks its genuine unsized declaration");
        }
        let growth = Growth::map::<ParameterId, Abilities>(0, 0, 1);
        charge(
            growth.work + 2,
            growth.bytes + size_of::<Self>() + size_of::<Parameters>() + 4 * size_of::<usize>(),
        )?;
        let mut own = HashMap::default();
        own.insert(id, Abilities::default());
        let mut result = self.as_ref().clone();
        result.parameters = Arc::new(Parameters {
            own,
            prefix: Some(Arc::clone(&self.parameters)),
        });
        Ok(Arc::new(result))
    }
}

#[cfg(test)]
impl Environment {
    pub(crate) const fn query_scope_layouts() -> (usize, usize, usize) {
        (
            size_of::<Self>(),
            size_of::<Parameters>(),
            size_of::<HashMap<ParameterId, Abilities>>(),
        )
    }
}

#[cfg(test)]
#[path = "../../../../tests/invariants/query_scopes.rs"]
mod tests;
#[cfg(test)]
impl Environment {
    pub(crate) fn check_query_scope_controls() -> Result<(), Box<dyn std::error::Error>> {
        tests::controls()
    }
}
