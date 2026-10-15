//! Deferred alias proofs form a typed dependency DAG; uses instantiate each scope once.
#[cfg(test)]
mod tests;
use super::traits::obligations::Obligations;
use crate::{
    resolve::model::DefId,
    types::{GenericArgs, ItemId, Store, substitute::Bound},
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
    direct: Vec<Obligations>,
    dependencies: Vec<Use>,
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
    ) -> Result<(), &'static str> {
        self.building(owner, parameters)?.direct.push(proof);
        Ok(())
    }

    pub(crate) fn dependency(
        &mut self,
        owner: DefId,
        parameters: DefId,
        dependency: Use,
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
            .push(dependency);
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
            });
        if contract.sealed || contract.parameters != parameters {
            return Err("cannot extend a completed alias obligation contract");
        }
        Ok(contract)
    }

    pub(crate) fn seal(&mut self, owner: DefId) {
        if let Some(contract) = self
            .data
            .as_mut()
            .and_then(|data| data.contracts.get_mut(&owner))
        {
            contract.sealed = true;
        }
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
        let mut pending = vec![use_];
        while let Some(use_) = pending.pop() {
            scope.charge(1)?;
            if !scope.retained.insert(use_) {
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
            for proof in &contract.direct {
                scope.charge(
                    proof
                        .equalities
                        .len()
                        .saturating_add(proof.outlives.len())
                        .saturating_add(proof.types.len()),
                )?;
                result.extend(proof.instantiate(store, &mut parameters, result.regions.end)?)?;
            }
            scope.charge(contract.dependencies.len())?;
            for dependency in contract.dependencies.iter().rev() {
                let arguments = store
                    .arguments(dependency.arguments)
                    .ok_or("missing alias dependency arguments")?;
                scope.charge(arguments.len())?;
                let arguments = arguments.collect::<Vec<_>>();
                let arguments = arguments
                    .into_iter()
                    .map(|argument| parameters.apply_argument(store, argument))
                    .collect::<Result<Vec<_>, _>>()?;
                pending.push(Use {
                    owner: dependency.owner,
                    arguments: store.intern_arguments(&arguments)?,
                });
            }
        }
        Ok(result)
    }
}
