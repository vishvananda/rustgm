//! Closed parameter defaults are checked independently; dependent defaults remain templates.
use super::{Checker, DefId, Error, NodeKey};
use crate::support::hash::HashMap;
use crate::{
    sema::nominals::Proof,
    types::{
        TypeId,
        arguments::Argument,
        parameters::{ParameterId, ParameterKind, clauses::Clause, uses::Sole},
        substitute,
    },
};
#[cfg(test)]
#[path = "../../../../tests/invariants/default_clauses.rs"]
mod tests;

impl Checker<'_> {
    pub(in super::super) fn trait_defaults(&mut self) -> Result<(), Error> {
        for index in 0..self.facts.associated.owners().len() {
            let owner = self.facts.associated.owners()[index];
            if let crate::sema::associated::OwnerKind::Trait { definition, .. } = owner.kind {
                let mut proof = self.declaration_proof(definition, None);
                self.parameter_defaults(definition, owner.source, &mut proof)?;
                self.retain_trait_proof(definition, owner.source, &mut proof);
            }
        }
        Ok(())
    }
    pub(in super::super) fn parameter_defaults(
        &mut self,
        owner: DefId,
        key: NodeKey,
        proof: &mut Proof,
    ) -> Result<(), Error> {
        let parameters = self
            .facts
            .generics
            .parameters
            .get(&owner)
            .map_or(&[][..], Vec::as_slice);
        if !parameters.iter().any(|p| p.default.is_some()) {
            return Ok(());
        }
        let count = parameters.len();
        let subjects = self
            .default_bound_subjects(owner)
            .map_err(|e| self.error(key, e))?;
        for index in 0..count {
            let parameter = self.facts.generics.parameters[&owner][index];
            let Some(default) = parameter.default else {
                continue;
            };
            let source = self.report.resolution.definitions[parameter.binding.0 as usize]
                .node
                .unwrap_or(key);
            let Some(id) = self.closed_parameter_default(parameter, source, proof)? else {
                continue;
            };
            let mut replace = substitute::Bound::parameter_only(&self.facts.types, id, default)
                .map_err(|e| self.error(source, e))?;
            for &index in subjects.get(&id).into_iter().flatten() {
                let predicate = self.facts.generics.predicates[&owner][index];
                let bound = replace
                    .clause(&mut self.facts.types, predicate.value)
                    .map_err(|e| self.error(source, e))?;
                if !check_requirement(&mut self.facts, proof, bound)
                    .map_err(|e| self.error(source, e))?
                {
                    return Err(self.error(
                        source,
                        "non-generic bound is not satisfied by a parameter default",
                    ));
                }
            }
        }
        Ok(())
    }
    fn closed_parameter_default(
        &mut self,
        parameter: crate::sema::generics::Parameter,
        source: NodeKey,
        proof: &mut Proof,
    ) -> Result<Option<ParameterId>, Error> {
        let Some(default) = parameter.default else {
            return Ok(None);
        };
        let id = crate::sema::generics::parameter_id(&self.facts.types, parameter.argument)
            .map_err(|e| self.error(source, e))?;
        match default {
            Argument::Const(constant) => {
                return Ok(matches!(
                    self.facts.types.const_kind(constant),
                    Some(crate::types::arguments::ConstKind::Value(_))
                )
                .then_some(id));
            }
            Argument::Type(ty) if self.closed_default(ty) => {
                let properties = proof
                    .check(&mut self.facts, ty)
                    .map_err(|e| self.error(source, e))?;
                let sized = self
                    .facts
                    .types
                    .parameter(id)
                    .is_some_and(|p| matches!(p.kind, ParameterKind::Type { sized: true }));
                if !properties.well_formed || (sized && !properties.sized) {
                    return Err(self.error(
                        source,
                        "parameter default must be well-formed and satisfy sizedness",
                    ));
                }
            }
            Argument::Type(_) => return Ok(None),
            Argument::Lifetime(_) => {
                return Err(self.error(source, "lifetimes cannot have defaults"));
            }
        }
        Ok(Some(id))
    }
    fn closed_default(&self, ty: TypeId) -> bool {
        self.facts.types.contains_parameters(ty) == Some(false)
            && self.facts.types.escaping_binders(ty) == Some(0)
    }
    fn default_bound_subjects(
        &self,
        owner: DefId,
    ) -> Result<HashMap<ParameterId, Vec<usize>>, &'static str> {
        let mut result: HashMap<ParameterId, Vec<usize>> = HashMap::default();
        let mut subjects = HashMap::default();
        let mut uses = Sole::default();
        for (index, predicate) in self
            .facts
            .generics
            .predicates
            .get(&owner)
            .into_iter()
            .flatten()
            .enumerate()
        {
            let original = self
                .facts
                .unreduced
                .predicate(predicate.source)
                .unwrap_or(predicate.value);
            if !original.checks_parameter_default(&self.facts.types)? {
                continue;
            }
            let id = if let Some(&id) = subjects.get(&predicate.value) {
                id
            } else {
                let id = uses.clause(&self.facts.types, predicate.value)?;
                subjects.insert(predicate.value, id);
                id
            };
            if let Some(id) = id {
                result.entry(id).or_default().push(index);
            }
        }
        Ok(result)
    }
}

fn check_requirement(
    facts: &mut crate::sema::model::Declarations,
    proof: &mut Proof,
    bound: Clause,
) -> Result<bool, &'static str> {
    if bound.contains_parameters(&facts.types)? || bound.escaping_binders(&facts.types)? != 0 {
        return Ok(true);
    }
    let subject = match bound {
        Clause::Predicate(value) => value.ty,
        Clause::ForAll(id) => {
            facts
                .types
                .quantified(id)
                .ok_or("invalid substituted default clause")?
                .predicate
                .ty
        }
    };
    // The written candidate contains no lifetimes. Its closed default can add
    // static lifetimes and nested callable binders, but no local clause slots.
    proof.check(facts, subject)?;
    proof.prove_clause(facts, bound)
}
