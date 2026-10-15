//! Deferred alias proofs form a typed dependency DAG; uses instantiate each scope once.
mod scopes;
#[cfg(test)]
mod tests;
use super::traits::obligations::Obligations;
use crate::{
    resolve::model::DefId,
    types::{GenericArgs, ItemId, Store, lifetime_proof::ScopeId, substitute::Bound},
};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Use {
    pub owner: DefId,
    pub arguments: GenericArgs,
}
struct Contract {
    parameters: DefId,
    sealed: bool,
    direct: Vec<(ScopeId, Obligations)>,
    dependencies: Vec<(ScopeId, Use)>,
    scopes: Option<Box<scopes::Scopes>>,
}
#[derive(Default)]
struct Data {
    contracts: HashMap<DefId, Contract>,
}
#[derive(Default)]
pub struct Requirements {
    data: Option<Box<Data>>,
}
#[derive(Default)]
pub(crate) struct Scope {
    retained: HashSet<Use>,
    failed: bool,
    steps: usize,
}

impl Scope {
    fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(count);
        if self.steps > 1_000_000 {
            return Err("alias obligation work budget exceeded");
        }
        Ok(())
    }
}
impl Requirements {
    #[must_use]
    pub fn contains(&self, owner: DefId) -> bool {
        self.data
            .as_ref()
            .is_some_and(|data| data.contracts.contains_key(&owner))
    }

    pub(crate) fn direct(
        &mut self,
        owner: DefId,
        parameters: DefId,
        proof: Obligations,
        scope: ScopeId,
    ) -> Result<(), &'static str> {
        self.building(owner, parameters)?
            .direct
            .push((scope, proof));
        Ok(())
    }

    pub(crate) fn dependency(
        &mut self,
        owner: DefId,
        parameters: DefId,
        dependency: Use,
        scope: ScopeId,
    ) -> Result<(), &'static str> {
        if !self
            .data
            .as_ref()
            .and_then(|data| data.contracts.get(&dependency.owner))
            .is_some_and(|contract| contract.sealed)
        {
            return Err("alias obligation dependency is not complete");
        }
        self.building(owner, parameters)?
            .dependencies
            .push((scope, dependency));
        Ok(())
    }

    fn building(&mut self, owner: DefId, parameters: DefId) -> Result<&mut Contract, &'static str> {
        let contract = self
            .data
            .get_or_insert_with(Default::default)
            .contracts
            .entry(owner)
            .or_insert_with(|| Contract {
                parameters,
                sealed: false,
                direct: Vec::new(),
                dependencies: Vec::new(),
                scopes: None,
            });
        if contract.sealed || contract.parameters != parameters {
            return Err("cannot extend a completed alias obligation contract");
        }
        Ok(contract)
    }

    pub(crate) fn seal(&mut self, owner: DefId) -> Result<(), &'static str> {
        if let Some(contract) = self
            .data
            .as_mut()
            .and_then(|data| data.contracts.get_mut(&owner))
        {
            if contract.scopes.as_ref().is_some_and(|s| !s.complete()) {
                return Err("alias proof scope is unfinished");
            }
            contract.sealed = true;
        }
        Ok(())
    }

    pub(crate) fn instantiate(
        &self,
        store: &mut Store,
        use_: Use,
        first: u32,
        scope: &mut Scope,
    ) -> Result<Obligations, &'static str> {
        if scope.failed {
            return Err("alias obligation scope has already failed");
        }
        let result = self.expand(store, use_, first, scope);
        scope.failed = result.is_err();
        result
    }

    fn expand(
        &self,
        store: &mut Store,
        use_: Use,
        first: u32,
        scope: &mut Scope,
    ) -> Result<Obligations, &'static str> {
        let mut result = Obligations::new(first);
        let mut pending = vec![(ScopeId::ROOT, use_)];
        let mut nested = HashSet::new();
        while let Some((caller, use_)) = pending.pop() {
            scope.charge(1)?;
            let fresh = if caller == ScopeId::ROOT {
                scope.retained.insert(use_)
            } else {
                nested.insert((caller, use_))
            };
            if !fresh {
                continue;
            }
            let contract = self
                .data
                .as_ref()
                .and_then(|data| data.contracts.get(&use_.owner))
                .filter(|contract| contract.sealed)
                .ok_or("missing completed alias obligation contract")?;
            let mut parameters =
                Bound::joint(store, ItemId(contract.parameters.0), use_.arguments)?;
            scope.charge(contract.scopes.as_ref().map_or(0, |s| s.len()))?;
            let mapping =
                scopes::Scopes::instantiate(contract.scopes.as_deref(), &mut result, caller)?;
            for &(local, ref proof) in &contract.direct {
                let (target, depth) = mapping.get(local)?;
                scope.charge(
                    proof
                        .equalities
                        .len()
                        .saturating_add(proof.outlives.len())
                        .saturating_add(proof.types.len()),
                )?;
                let (proof, work) =
                    proof.instantiate_at(store, &mut parameters, result.regions.end, depth)?;
                scope.charge(work)?;
                result.append_at(proof, target)?;
            }
            scope.charge(contract.dependencies.len())?;
            for &(local, dependency) in contract.dependencies.iter().rev() {
                let (target, depth) = mapping.get(local)?;
                let arguments = store
                    .arguments(dependency.arguments)
                    .ok_or("missing alias dependency arguments")?;
                scope.charge(arguments.len())?;
                let arguments = arguments.collect::<Vec<_>>();
                let arguments = arguments
                    .into_iter()
                    .map(|argument| parameters.apply_argument_inside(store, argument, depth))
                    .collect::<Result<Vec<_>, _>>()?;
                pending.push((
                    target,
                    Use {
                        owner: dependency.owner,
                        arguments: store.intern_arguments(&arguments)?,
                    },
                ));
            }
            scope.charge(parameters.steps)?;
        }
        Ok(result)
    }
}
