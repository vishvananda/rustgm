//! Closed nominal defaults are checked independently; dependent defaults remain templates.
use super::{Checker, DefId, Error};
use crate::{
    sema::nominals::Proof,
    types::{
        Kind, TypeId,
        arguments::Argument,
        parameters::{ParameterId, ParameterKind, predicates::Bound, uses::Uses},
        substitute,
    },
};
use std::collections::HashMap;

impl Checker<'_> {
    pub(super) fn nominal_defaults(&mut self) -> Result<(), Error> {
        let mut proof = Proof::new(None);
        for index in 0..self.nominals.entries.len() {
            let item = self.nominals.entries[index].item;
            let key = self.nominals.entries[index].key;
            let owner = DefId(item.0);
            let parameters = self
                .facts
                .generics
                .parameters
                .get(&owner)
                .map_or(&[][..], Vec::as_slice);
            if !parameters.iter().any(|p| p.default.is_some()) {
                continue;
            }
            let count = parameters.len();
            let subjects = self
                .default_bound_subjects(owner)
                .map_err(|e| self.error(key, e))?;
            for index in 0..count {
                let parameter = self.facts.generics.parameters[&owner][index];
                let Some(Argument::Type(default)) = parameter.default else {
                    continue;
                };
                if !self.closed_default(default) {
                    continue;
                }
                let source = self.report.resolution.definitions[parameter.binding.0 as usize]
                    .node
                    .unwrap_or(key);
                let Argument::Type(formal) = parameter.argument else {
                    return Err(self.error(source, "invalid default parameter kind"));
                };
                let Some(Kind::Parameter(id)) = self.facts.types.kind(formal) else {
                    return Err(self.error(source, "missing default parameter identity"));
                };
                let properties = proof
                    .check(&mut self.facts, default)
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
                let mut replace = substitute::Bound::parameter_only(
                    &self.facts.types,
                    id,
                    Argument::Type(default),
                )
                .map_err(|e| self.error(source, e))?;
                for &index in subjects.get(&id).into_iter().flatten() {
                    let predicate = self.facts.generics.predicates[&owner][index];
                    let ty = replace
                        .apply(&mut self.facts.types, predicate.value.ty)
                        .map_err(|e| self.error(source, e))?;
                    if self.closed_default(ty) {
                        let bound = crate::types::parameters::predicates::Predicate {
                            ty,
                            ..predicate.value
                        };
                        proof
                            .check(&mut self.facts, ty)
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
        }
        Ok(())
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
        let mut subjects: HashMap<TypeId, Option<ParameterId>> = HashMap::new();
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
                || self.facts.types.escaping_binders(predicate.value.ty) != Some(0)
            {
                continue;
            }
            let id = if let Some(&id) = subjects.get(&predicate.value.ty) {
                id
            } else {
                let mut uses = Uses::default();
                uses.collect(&self.facts.types, predicate.value.ty)?;
                steps = steps.saturating_add(uses.steps);
                if steps > 1_000_000 {
                    return Err("default bound subject work budget exceeded");
                }
                let id = if uses.parameters.len() == 1 {
                    uses.parameters.iter().next().copied()
                } else {
                    None
                };
                subjects.insert(predicate.value.ty, id);
                id
            };
            if let Some(id) = id {
                result.entry(id).or_default().push(index);
            }
        }
        Ok(result)
    }
}
