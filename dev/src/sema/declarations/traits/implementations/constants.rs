//! Retain normalized member shapes and discharge their contracts during impl validation.
use super::{Checker, DefId, Error, NodeKey};
use crate::{
    resolve::{expansion::Report, model::DefKind, names::NameId},
    sema::{model::Declarations, traits::conformance::Definitions},
    syntax::item::ItemKind,
    types::{
        TypeId,
        infer::Context,
        lifetime_proof::Obligations,
        outlives::Queries,
        parameters::predicates::{Bound, Predicate},
    },
};
use std::collections::HashMap;

/// Normalized shape and formation debts are retained until nominal facts exist.
pub(super) struct Contract {
    definition: DefId,
    source: NodeKey,
    types: [TypeId; 2],
    evidence: Obligations,
}

#[derive(Default)]
pub(super) struct Members {
    owners: HashMap<DefId, Vec<DefId>>,
    names: HashMap<(DefId, NameId), DefId>,
}
impl Members {
    pub(super) fn selected(
        &self,
        facts: &Declarations,
        report: &Report,
        owner: DefId,
        declaration: DefId,
    ) -> Result<Option<DefId>, &'static str> {
        let owner = facts
            .associated
            .declaration_owner(owner)
            .ok_or("missing constant implementation owner")?;
        let name = report.resolution.definitions[declaration.0 as usize].name;
        Ok(self.names.get(&(owner, name)).copied())
    }
    pub(super) fn new(report: &Report, facts: &Declarations) -> Self {
        let mut result = Self::default();
        for (member, owner) in facts.associated.declaration_members() {
            let definition = &report.resolution.definitions[member.0 as usize];
            if definition.kind == DefKind::Const {
                result.owners.entry(owner).or_default().push(member);
                result.names.insert((owner, definition.name), member);
            }
        }
        result
    }
}

impl Checker<'_> {
    pub(super) fn trait_impl_constants(
        &mut self,
        owner: DefId,
        source: NodeKey,
        predicate: Predicate,
        members: &Members,
        definitions: &Definitions,
    ) -> Result<Vec<(DefId, Option<DefId>)>, Error> {
        let Bound::Trait(id) = predicate.bound else {
            return Err(self.error(source, "missing constant impl trait predicate"));
        };
        let app = self
            .facts
            .types
            .trait_application(id)
            .ok_or_else(|| self.error(source, "missing constant impl trait application"))?;
        let trait_owner = self
            .facts
            .associated
            .declaration_owner(DefId(app.item.0))
            .ok_or_else(|| self.error(source, "missing constant trait owner"))?;
        let impl_owner = self
            .facts
            .associated
            .declaration_owner(owner)
            .ok_or_else(|| self.error(source, "missing constant impl owner"))?;
        for &actual in members.owners.get(&impl_owner).into_iter().flatten() {
            let definition = &self.report.resolution.definitions[actual.0 as usize];
            if !members.names.contains_key(&(trait_owner, definition.name)) {
                return Err(self.error(
                    definition.node.unwrap_or(source),
                    "implementation constant is not a member of the trait",
                ));
            }
        }
        let mut result = Vec::new();
        let mut substitution = None;
        for &expected in members.owners.get(&trait_owner).into_iter().flatten() {
            let declaration = &self.report.resolution.definitions[expected.0 as usize];
            if let Some(&actual) = members.names.get(&(impl_owner, declaration.name)) {
                if substitution.is_none() {
                    substitution = Some(
                        crate::sema::traits::results::substitution(&self.facts, predicate)
                            .map_err(|e| self.error(source, e))?,
                    );
                }
                let folder = substitution
                    .as_mut()
                    .ok_or_else(|| self.error(source, "missing constant contract substitution"))?;
                self.constant_conformance(owner, source, (expected, actual), folder, definitions)?;
                result.push((expected, Some(actual)));
            } else {
                let key = declaration
                    .node
                    .ok_or_else(|| self.error(source, "missing trait constant source"))?;
                if !matches!(self.item(key)?.kind, ItemKind::Const { value: Some(_), .. }) {
                    return Err(self.error(
                        source,
                        "implementation is missing a required associated constant",
                    ));
                }
                result.push((expected, None));
            }
        }
        Ok(result)
    }

    pub(super) fn constant_conformance(
        &mut self,
        owner: DefId,
        source: NodeKey,
        pair: (DefId, DefId),
        substitution: &mut crate::types::substitute::Bound,
        definitions: &Definitions,
    ) -> Result<(), Error> {
        let request = crate::sema::traits::implementations::ConstantRequest {
            owner,
            declaration: pair.0,
        };
        let contract = if let Some(contract) = self.constant_members.contracts.remove(&request) {
            if contract.definition != pair.1 {
                return Err(self.error(source, "constant contract changed its selected definition"));
            }
            contract
        } else {
            self.constant_contract(owner, source, pair, substitution, definitions)?
        };
        self.prove_constant_contract(owner, contract)
    }

    pub(super) fn constant_contract(
        &mut self,
        owner: DefId,
        source: NodeKey,
        pair: (DefId, DefId),
        substitution: &mut crate::types::substitute::Bound,
        definitions: &Definitions,
    ) -> Result<Contract, Error> {
        let (expected, definition) = pair;
        let key = self.report.resolution.definitions[definition.0 as usize]
            .node
            .unwrap_or(source);
        let expected = self.facts.definitions[expected.0 as usize]
            .ok_or_else(|| self.error(key, "missing trait constant type"))?;
        let mut actual = self.facts.definitions[definition.0 as usize]
            .ok_or_else(|| self.error(key, "missing implementation constant type"))?;
        if !self.facts.associated.families.is_empty() {
            let ItemKind::Const { ty: annotation, .. } = self.item(key)?.kind else {
                return Err(self.error(key, "missing implementation constant annotation"));
            };
            // Families require their complete written application here. Ordinary
            // annotations retain their existing declaration proof and diagnostic.
            actual = self
                .facts
                .unreduced
                .node(NodeKey {
                    node: annotation,
                    ..key
                })
                .unwrap_or(actual);
        }
        let expected = substitution
            .apply(&mut self.facts.types, expected)
            .map_err(|e| self.error(key, e))?;
        let mut types = [expected, actual];
        let mut evidence = Obligations::new(1);
        let environment = self.facts.generics.environments.get(&owner).cloned();
        definitions
            .values(&mut self.facts, environment, &mut evidence, &mut types)
            .map_err(|e| self.error(key, e))?;
        let [expected, actual] = types;
        if expected != actual {
            let mut shape = Context::default();
            let a = shape
                .known(&self.facts.types, actual)
                .map_err(|e| self.error(key, e.to_string()))?;
            let b = shape
                .known(&self.facts.types, expected)
                .map_err(|e| self.error(key, e.to_string()))?;
            shape
                .equate_shape(&self.facts.types, a, b)
                .map_err(|e| self.error(key, e.to_string()))?;
        }
        Ok(Contract {
            definition,
            source: key,
            types,
            evidence,
        })
    }

    fn prove_constant_contract(&self, owner: DefId, contract: Contract) -> Result<(), Error> {
        let Contract {
            source: key,
            types: [expected, actual],
            evidence,
            ..
        } = contract;
        // Normalization can erase inputs: identical results still owe their
        // retained formation proof. Only a genuinely empty contract is complete.
        if expected == actual && evidence.is_empty() {
            return Ok(());
        }
        let mut queries = Queries::new(
            &self.facts.types,
            &self.facts.lifetimes.nominal_requirements,
            &self.facts.lifetimes.callables,
        );
        let mut constraints = self
            .trait_region_environment(owner, &mut queries)
            .map_err(|e| self.error(key, e))?;
        evidence
            .require(&mut constraints, &mut queries)
            .map_err(|e| self.error(key, e))?;
        constraints
            .coerce(actual, expected, false, 0)
            .map_err(|e| self.error(key, e.to_string()))?;
        constraints.solve().map_err(|e| {
            self.error(
                key,
                format!("implementation constant type does not satisfy trait: {e}"),
            )
        })?;
        Ok(())
    }
}
