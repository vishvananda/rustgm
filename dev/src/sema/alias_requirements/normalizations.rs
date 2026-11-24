//! Deferred type formation remains a requirement even if an outer alias erases its result.
use super::{Contract, DefId, Obligations, Requirements, Scope, scopes::Mapping};
use crate::types::{Store, TypeId, lifetime_proof::ScopeId, substitute::Bound};

pub(crate) struct Expansion {
    pub proof: Obligations,
    pub normalizations: Vec<(ScopeId, TypeId)>,
    pub sized: Vec<(ScopeId, TypeId)>,
}
impl Expansion {
    pub(super) fn new(first: u32) -> Self {
        Self {
            proof: Obligations::new(first),
            normalizations: Vec::new(),
            sized: Vec::new(),
        }
    }
    pub(super) fn instantiate_normalizations(
        &mut self,
        store: &mut Store,
        contract: &Contract,
        mapping: &Mapping,
        parameters: &mut Bound,
        scope: &mut Scope,
    ) -> Result<(), &'static str> {
        scope.charge(contract.normalizations.len())?;
        for &(local, ty) in &contract.normalizations {
            let (target, depth) = mapping.get(local)?;
            let ty = parameters.apply_inside(store, ty, depth)?;
            if target != ScopeId::ROOT || scope.normalized.insert(ty) {
                self.normalizations.push((target, ty));
            }
        }
        scope.charge(contract.sized.len())?;
        for &(local, ty) in &contract.sized {
            let (target, depth) = mapping.get(local)?;
            let ty = parameters.apply_inside(store, ty, depth)?;
            if target != ScopeId::ROOT || scope.sized.insert(ty) {
                self.sized.push((target, ty));
            }
        }
        Ok(())
    }
}
impl Requirements {
    /// A source alias can reconstruct a single deferred root from its type
    /// recipe when the completed contract has no other proof work.
    pub(crate) fn source_target(&self, owner: DefId, target: TypeId, projected: bool) -> bool {
        let contract = self
            .data
            .as_ref()
            .and_then(|data| data.contracts.get(&owner));
        match (projected, contract) {
            (false, None) => true,
            (true, Some(contract)) => {
                contract.parameters == owner
                    && contract.sealed
                    && contract.direct.is_empty()
                    && contract.dependencies.is_empty()
                    && contract.sized.is_empty()
                    && contract.scopes.is_none()
                    && contract.normalizations == [(ScopeId::ROOT, target)]
            }
            _ => false,
        }
    }

    pub(crate) const fn is_empty(&self) -> bool {
        self.data.is_none()
    }

    pub(crate) fn source_normalizations(
        &self,
        owner: DefId,
    ) -> Result<&[(ScopeId, TypeId)], &'static str> {
        let contract = self
            .data
            .as_ref()
            .and_then(|data| data.contracts.get(&owner))
            .filter(|contract| {
                contract.parameters == owner
                    && contract.sealed
                    && contract.direct.is_empty()
                    && contract.dependencies.is_empty()
                    && contract.sized.is_empty()
                    && contract.scopes.is_none()
                    && !contract.normalizations.is_empty()
            })
            .ok_or("source alias has deferred proof inputs beyond its target recipe")?;
        Ok(&contract.normalizations)
    }

    pub(crate) fn source_projection_allocation(
        &self,
        count: usize,
    ) -> crate::support::retained::Allocation {
        use crate::support::retained::{Allocation, Growth};
        let (len, capacity) = self.data.as_ref().map_or((0, 0), |data| {
            (data.contracts.len(), data.contracts.capacity())
        });
        let growth = Growth::map::<DefId, Contract>(len, capacity, 1);
        Allocation {
            bytes: growth
                .bytes
                .saturating_add(usize::from(self.is_empty()) * size_of::<super::Data>())
                .saturating_add(count.saturating_mul(4 * size_of::<(ScopeId, TypeId)>())),
            visits: growth.work.saturating_add(count),
        }
    }

    pub(crate) fn source_projection(
        &mut self,
        owner: DefId,
        target: TypeId,
    ) -> Result<(), &'static str> {
        self.normalization(owner, owner, target, ScopeId::ROOT)?;
        self.seal(owner)
    }

    pub(crate) fn sized(
        &mut self,
        owner: DefId,
        parameters: DefId,
        ty: TypeId,
        scope: ScopeId,
    ) -> Result<(), &'static str> {
        self.building(owner, parameters)?.sized.push((scope, ty));
        Ok(())
    }
    pub(crate) fn normalization(
        &mut self,
        owner: DefId,
        parameters: DefId,
        ty: TypeId,
        scope: ScopeId,
    ) -> Result<(), &'static str> {
        self.building(owner, parameters)?
            .normalizations
            .push((scope, ty));
        Ok(())
    }
}
