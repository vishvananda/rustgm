//! Trait paths use ordinary argument elaboration, with Self supplied by the bound subject.
use super::{Checker, DefId, Error, NodeKey, Type, TypeId, regions};
use crate::types::{
    ItemId,
    arguments::Argument,
    substitute::Bound,
    traits::{Application, ApplicationId},
};

impl Checker<'_> {
    pub(super) fn retain_trait_contract(
        &mut self,
        owner: crate::sema::associated::Owner,
        definition: DefId,
    ) -> Result<(), Error> {
        let schema = self
            .facts
            .associated
            .schema(definition)
            .ok_or_else(|| self.error(owner.source, "missing trait Self schema"))?;
        let header = crate::sema::traits::Header {
            self_type: schema.self_type,
            parameters: self.facts.generics.parameters[&definition].clone(),
            predicates: self
                .facts
                .generics
                .predicates
                .get(&definition)
                .cloned()
                .unwrap_or_default(),
            regions: schema.bounds.clone(),
        };
        self.facts
            .generics
            .traits
            .get_or_insert_with(Default::default)
            .declare(ItemId(definition.0), header)
            .map_err(|e| self.error(owner.source, e))
    }

    pub(super) fn complete_trait_contracts(&mut self) -> Result<(), Error> {
        if let Some(contracts) = &self.facts.generics.traits {
            contracts
                .check_cycles(&self.facts.types)
                .map_err(|(item, error)| {
                    let key = self.report.resolution.definitions[item.0 as usize].node;
                    key.map_or_else(
                        || Error::at(crate::resolve::unit::FileId(0), 0, error),
                        |key| self.error(key, error),
                    )
                })?;
        }
        for index in 0..self.facts.associated.owners().len() {
            let owner = self.facts.associated.owners()[index];
            if let crate::sema::associated::OwnerKind::Trait { definition, .. } = owner.kind {
                self.facts
                    .generics
                    .complete_environment(&mut self.facts.types, definition)
                    .map_err(|e| self.error(owner.source, e))?;
            }
        }
        Ok(())
    }

    pub(super) fn trait_requirements(
        &mut self,
        predicate: crate::sema::generics::Predicate,
        proof: &mut crate::sema::nominals::Proof,
    ) -> Result<(), Error> {
        let requirements = self
            .facts
            .generics
            .traits
            .as_mut()
            .ok_or_else(|| Error::at(predicate.source.file, 0, "missing trait contracts"))?
            .requirements(&mut self.facts.types, predicate.value)
            .map_err(|e| self.error(predicate.source, e))?;
        for &ty in &requirements.types {
            if !proof
                .check(&mut self.facts, ty)
                .map_err(|e| self.error(predicate.source, e))?
                .well_formed
            {
                return Err(self.error(predicate.source, "trait argument must be well-formed"));
            }
        }
        for &requirement in &requirements.required {
            if matches!(
                requirement.bound,
                crate::types::parameters::predicates::Bound::Outlives(_)
            ) {
                continue; // Proved with the completed nominal lifetime contracts.
            }
            if !proof
                .prove(&mut self.facts, requirement)
                .map_err(|e| self.error(predicate.source, e))?
            {
                return Err(self.error(
                    predicate.source,
                    "trait argument does not satisfy its declaration requirements",
                ));
            }
        }
        Ok(())
    }

    pub(super) fn trait_path(
        &mut self,
        path: NodeKey,
        owner: DefId,
        subject: TypeId,
        context: regions::Context,
    ) -> Result<ApplicationId, Error> {
        let self_type = self
            .facts
            .associated
            .schema(owner)
            .ok_or_else(|| {
                self.error(path, "trait bound requires an elaborated trait declaration")
            })?
            .self_type;
        let Some(Type::Parameter(self_parameter)) = self.facts.types.kind(self_type) else {
            return Err(self.error(path, "trait Self must be a rigid type parameter"));
        };
        let arguments = self.application_arguments(path, owner, context)?;
        let mut replace =
            Bound::parameter_only(&self.facts.types, self_parameter, Argument::Type(subject))
                .map_err(|e| self.error(path, e))?;
        let mut arguments = self
            .facts
            .types
            .arguments(arguments)
            .ok_or_else(|| self.error(path, "missing elaborated trait arguments"))?
            .collect::<Vec<_>>();
        for argument in &mut arguments {
            *argument = replace
                .apply_argument(&mut self.facts.types, *argument)
                .map_err(|e| self.error(path, e))?;
        }
        let arguments = self
            .facts
            .types
            .intern_arguments(&arguments)
            .map_err(|e| self.error(path, e))?;
        self.facts
            .types
            .intern_trait_application(Application {
                item: ItemId(owner.0),
                arguments,
            })
            .map_err(|e| self.error(path, e))
    }
}
