//! Closed parameter defaults are checked independently; dependent defaults remain templates.
use super::{Checker, DefId, Error, NodeKey};
use crate::{
    sema::nominals::Proof,
    types::{
        TypeId,
        arguments::Argument,
        parameters::{ParameterId, ParameterKind, predicates::Bound, uses::Uses},
        substitute,
    },
};
use std::collections::HashMap;

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
                    .predicate(&mut self.facts.types, predicate.value)
                    .map_err(|e| self.error(source, e))?;
                if !bound
                    .contains_parameters(&self.facts.types)
                    .map_err(|e| self.error(source, e))?
                    && bound
                        .escaping_binders(&self.facts.types)
                        .map_err(|e| self.error(source, e))?
                        == 0
                {
                    proof
                        .check(&mut self.facts, bound.ty)
                        .map_err(|e| self.error(source, e))?;
                    if !proof
                        .prove(&mut self.facts, bound)
                        .map_err(|e| self.error(source, e))?
                    {
                        return Err(self.error(
                            source,
                            "non-generic bound is not satisfied by a parameter default",
                        ));
                    }
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
        let mut result: HashMap<ParameterId, Vec<usize>> = HashMap::new();
        let mut subjects = HashMap::new();
        let mut steps = 0_usize;
        for (index, predicate) in self
            .facts
            .generics
            .predicates
            .get(&owner)
            .into_iter()
            .flatten()
            .enumerate()
        {
            if matches!(predicate.value.bound, Bound::Outlives(_))
                || predicate.value.escaping_binders(&self.facts.types)? != 0
            {
                continue;
            }
            let id = if let Some(&id) = subjects.get(&predicate.value) {
                id
            } else {
                let mut uses = Uses::default();
                uses.predicate(&self.facts.types, predicate.value)?;
                steps = steps.saturating_add(uses.steps);
                if steps > 1_000_000 {
                    return Err("default bound subject work budget exceeded");
                }
                let id = if uses.parameters.len() + uses.constants.len() == 1 {
                    uses.parameters
                        .iter()
                        .chain(&uses.constants)
                        .next()
                        .copied()
                } else {
                    None
                };
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
