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
