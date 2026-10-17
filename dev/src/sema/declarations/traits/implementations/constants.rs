//! Match constant declarations once and retain override identity after type conformance.
use super::{Checker, DefId, Error, NodeKey};
use crate::{
    resolve::{expansion::Report, model::DefKind, names::NameId},
    sema::{associated::OwnerId, model::Declarations, traits::conformance::Definitions},
    syntax::item::ItemKind,
    types::{
        infer::Context,
        lifetime_proof::Obligations,
        outlives::Queries,
        parameters::predicates::{Bound, Predicate},
    },
};
use std::collections::HashMap;

#[derive(Default)]
pub(super) struct Members {
    owners: HashMap<OwnerId, Vec<DefId>>,
    names: HashMap<(OwnerId, NameId), DefId>,
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
            .owner(owner)
            .ok_or("missing constant implementation owner")?;
        let name = report.resolution.definitions[declaration.0 as usize].name;
        Ok(self.names.get(&(owner, name)).copied())
    }
    pub(super) fn new(report: &Report, facts: &Declarations) -> Self {
        let mut result = Self::default();
        for member in facts.associated.members() {
            let definition = &report.resolution.definitions[member.definition.0 as usize];
            if definition.kind == DefKind::Const {
                result
                    .owners
                    .entry(member.owner)
                    .or_default()
                    .push(member.definition);
                result
                    .names
                    .insert((member.owner, definition.name), member.definition);
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
            .owner(DefId(app.item.0))
            .ok_or_else(|| self.error(source, "missing constant trait owner"))?;
        let impl_owner = self
            .facts
            .associated
            .owner(owner)
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
        let (expected, actual) = pair;
        let key = self.report.resolution.definitions[actual.0 as usize]
            .node
            .unwrap_or(source);
        let expected = self.facts.definitions[expected.0 as usize]
            .ok_or_else(|| self.error(key, "missing trait constant type"))?;
        let actual = self.facts.definitions[actual.0 as usize]
            .ok_or_else(|| self.error(key, "missing implementation constant type"))?;
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
        // Exact canonical equality needs no coercion environment. Any erased
        // projection debt still forces the ordinary lifetime proof below.
        if expected == actual && evidence.is_empty() {
            return Ok(());
        }
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
