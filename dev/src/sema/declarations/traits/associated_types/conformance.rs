//! Match each associated type once, then prove its definition in the impl environment.
use super::{Checker, DefId, Error, NodeKey};
use crate::{
    resolve::{expansion::Report, model::DefKind, names::NameId},
    sema::{associated::OwnerId, model::Declarations},
    types::{
        outlives::Queries,
        parameters::predicates::{Bound, Predicate},
    },
};
use std::collections::HashMap;

#[derive(Default)]
pub(in super::super) struct Members {
    owners: HashMap<OwnerId, Vec<DefId>>,
    names: HashMap<(OwnerId, NameId), DefId>,
}
impl Members {
    pub(in super::super) fn new(report: &Report, facts: &Declarations) -> Self {
        let mut result = Self::default();
        for member in facts.associated.members() {
            let definition = &report.resolution.definitions[member.definition.0 as usize];
            if definition.kind == DefKind::TypeAlias {
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
    pub(in crate::sema::declarations::traits) fn trait_impl_types(
        &mut self,
        owner: DefId,
        source: NodeKey,
        predicate: Predicate,
        members: &Members,
    ) -> Result<Vec<(DefId, DefId)>, Error> {
        let Bound::Trait(id) = predicate.bound else {
            return Err(self.error(source, "missing impl trait predicate"));
        };
        let app = self
            .facts
            .types
            .trait_application(id)
            .ok_or_else(|| self.error(source, "missing impl trait application"))?;
        let trait_owner = self
            .facts
            .associated
            .owner(DefId(app.item.0))
            .ok_or_else(|| self.error(source, "missing trait owner"))?;
        let impl_owner = self
            .facts
            .associated
            .owner(owner)
            .ok_or_else(|| self.error(source, "missing impl owner"))?;
        if !members.owners.contains_key(&trait_owner) && !members.owners.contains_key(&impl_owner) {
            return Ok(Vec::new());
        }
        for &actual in members.owners.get(&impl_owner).into_iter().flatten() {
            let definition = &self.report.resolution.definitions[actual.0 as usize];
            if !members.names.contains_key(&(trait_owner, definition.name)) {
                return Err(self.error(
                    definition.node.unwrap_or(source),
                    "implementation type is not a member of the trait",
                ));
            }
        }
        let environment = self.facts.generics.environments.get(&owner).cloned();
        let mut proof = self.declaration_proof(owner, environment);
        let mut result = Vec::new();
        let mut substitution: Option<crate::types::substitute::Bound> = None;
        for &expected in members.owners.get(&trait_owner).into_iter().flatten() {
            let name = self.report.resolution.definitions[expected.0 as usize].name;
            let &actual = members.names.get(&(impl_owner, name)).ok_or_else(|| {
                self.error(
                    source,
                    "implementation is missing a required associated type",
                )
            })?;
            let key = self.report.resolution.definitions[actual.0 as usize]
                .node
                .ok_or_else(|| self.error(source, "missing associated type definition"))?;
            let ty = self.facts.definitions[actual.0 as usize]
                .ok_or_else(|| self.error(key, "missing associated type definition type"))?;
            if !proof
                .check(&mut self.facts, ty)
                .map_err(|e| self.error(key, e))?
                .well_formed
            {
                return Err(self.error(key, "associated type definition must be well-formed"));
            }
            for requirement in
                self.associated_result_bounds(key, predicate, expected, ty, &mut substitution)?
            {
                if !proof
                    .prove(&mut self.facts, requirement)
                    .map_err(|e| self.error(key, e))?
                {
                    return Err(self.error(
                        key,
                        "associated type definition does not satisfy its declared bounds",
                    ));
                }
            }
            // The mapping is published only with the rest of the checked implementation.
            result.push((expected, actual));
        }
        self.associated_type_regions(owner, source, predicate, &result)?;
        self.retain_trait_proof(owner, source, &mut proof);
        Ok(result)
    }

    /// Instantiate result contracts with one shared mapping, leaving actual types in impl scope.
    fn associated_result_bounds(
        &mut self,
        key: NodeKey,
        predicate: Predicate,
        expected: DefId,
        ty: crate::types::TypeId,
        substitution: &mut Option<crate::types::substitute::Bound>,
    ) -> Result<Vec<Predicate>, Error> {
        let contract = self
            .facts
            .associated
            .type_contract(expected)
            .ok_or_else(|| self.error(key, "missing associated type contract"))?;
        let projection = contract.projection;
        let mut required = contract.bounds.clone();
        for requirement in &mut required {
            let direct = requirement.ty == projection;
            if !direct || !matches!(requirement.bound, Bound::Copy | Bound::Clone | Bound::Sized) {
                if substitution.is_none() {
                    *substitution = Some(
                        result_substitution(&self.facts, predicate)
                            .map_err(|e| self.error(key, e))?,
                    );
                }
                if direct {
                    // The already elaborated result belongs to the impl's scope.
                    // Map only the trait's template, never the actual type twice.
                    requirement.ty = crate::types::TypeId::UNIT;
                }
                *requirement = substitution
                    .as_mut()
                    .ok_or_else(|| {
                        Error::at(key.file, 0, "missing associated result substitution")
                    })?
                    .predicate(&mut self.facts.types, *requirement)
                    .map_err(|e| self.error(key, e))?;
            }
            if direct {
                requirement.ty = ty;
            }
        }
        Ok(required)
    }

    fn associated_type_regions(
        &self,
        owner: DefId,
        source: NodeKey,
        predicate: Predicate,
        definitions: &[(DefId, DefId)],
    ) -> Result<(), Error> {
        let mut inputs = crate::types::bound_uses::Uses::constraining();
        inputs
            .predicate(&self.facts.types, predicate)
            .map_err(|e| self.error(source, e))?;
        let mut outputs = crate::types::bound_uses::Uses::default();
        let mut queries = Queries::new(
            &self.facts.types,
            &self.facts.lifetimes.nominal_requirements,
            &self.facts.lifetimes.callables,
        );
        let mut constraints = self
            .trait_region_environment(owner, &mut queries)
            .map_err(|e| self.error(source, e))?;
        for &(_, definition) in definitions {
            let key = self.report.resolution.definitions[definition.0 as usize]
                .node
                .ok_or_else(|| self.error(source, "missing associated type source"))?;
            let ty = self.facts.definitions[definition.0 as usize]
                .ok_or_else(|| self.error(key, "missing associated type definition type"))?;
            outputs
                .collect(
                    &self.facts.types,
                    self.facts.unreduced.definition(definition).unwrap_or(ty),
                )
                .map_err(|e| self.error(key, e))?;
            if !outputs.indices.is_subset(&inputs.indices) {
                return Err(self.error(key, "associated type uses an impl lifetime not constrained by Self or the trait arguments"));
            }
            let required = queries
                .symbolic_requirements(ty)
                .map_err(|e| self.error(key, e))?;
            for &(a, b) in required.regions {
                constraints
                    .outlives(a, b, 0)
                    .map_err(|e| self.error(key, e.to_string()))?;
            }
            for &(parameter, region) in required.parameters {
                constraints
                    .parameter_outlives(parameter, region, 0)
                    .map_err(|e| self.error(key, e.to_string()))?;
            }
        }
        constraints.solve().map(|_| ()).map_err(|e| {
            self.error(
                source,
                format!("associated type lifetime requirement is not satisfied: {e}"),
            )
        })
    }
}

fn result_substitution(
    facts: &Declarations,
    predicate: Predicate,
) -> Result<crate::types::substitute::Bound, &'static str> {
    use crate::{sema::generics::parameter_id, types::arguments::Argument};
    let Bound::Trait(id) = predicate.bound else {
        return Err("missing associated result trait");
    };
    let application = facts
        .types
        .trait_application(id)
        .ok_or("missing associated result application")?;
    let owner = DefId(application.item.0);
    let schema = facts
        .associated
        .schema(owner)
        .ok_or("missing associated result schema")?;
    let parameters = facts
        .generics
        .parameters
        .get(&owner)
        .map_or(&[][..], Vec::as_slice);
    let arguments = facts
        .types
        .arguments(application.arguments)
        .ok_or("missing associated result arguments")?;
    if parameters.len() != arguments.len() {
        return Err("associated result argument arity mismatch");
    }
    let mut mapping = vec![(
        parameter_id(&facts.types, Argument::Type(schema.self_type))?,
        Argument::Type(predicate.ty),
    )];
    let mut regions = Vec::new();
    for (formal, actual) in parameters.iter().zip(arguments) {
        if let Argument::Lifetime(_) = formal.argument {
            let Argument::Lifetime(region) = actual else {
                return Err("associated result lifetime argument kind mismatch");
            };
            regions.push(region);
        } else {
            mapping.push((parameter_id(&facts.types, formal.argument)?, actual));
        }
    }
    if regions.len() != schema.captures as usize {
        return Err("associated result lifetime arity mismatch");
    }
    crate::types::substitute::Bound::parameters_captured(&facts.types, &mapping, regions)
}
