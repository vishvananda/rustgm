//! Const Drop declaration fields retain the actual impl's conditional premises.
use super::coverage::NominalContract;
use crate::{
    sema::{declarations::Checker, traits::obligations::Obligations},
    support::bindings::LangItem,
    types::{
        GenericArgs, ItemId, TypeId,
        parameters::predicates::{Bound, Predicate},
        traits::Application,
    },
};
impl Checker<'_> {
    pub(super) fn validate_const_drop_fields(
        &mut self,
        contract: &NominalContract,
        target: TypeId,
        first: u32,
    ) -> Result<Option<Obligations>, super::super::Error> {
        if !self.const_drop_coverage_owner(contract.owner, contract.item, contract.scope)
            || self.facts.source_bindings.is_none()
        {
            return Ok(None);
        }
        let key = contract.source;
        let (types, work) = {
            let mut fields = self
                .facts
                .aggregates
                .semantic_fields(&self.facts.types, target)
                .map_err(|e| self.error(key, e))?;
            let mut types = Vec::new();
            while let Some(ty) = fields
                .next(&mut self.facts.types)
                .map_err(|e| self.error(key, e))?
            {
                fields
                    .reserve(&mut types, 1)
                    .map_err(|e| self.error(key, e))?;
                types.push(ty);
            }
            (types, fields.work())
        };
        self.facts.body_work.canonical_steps =
            self.facts.body_work.canonical_steps.saturating_add(work);
        if types.is_empty() {
            return Ok(None);
        }
        let catalog = self
            .facts
            .source_bindings
            .as_deref()
            .ok_or_else(|| self.error(key, "const Drop field catalog disappeared"))?;
        catalog
            .validated_language(LangItem::Drop)
            .map_err(|e| self.error(key, e))?
            .ok_or_else(|| self.error(key, "const Drop field validation lost actual Drop"))?;
        let role = catalog
            .validated_language(LangItem::Destruct)
            .map_err(|e| self.error(key, e))?
            .ok_or_else(|| self.error(key, "const Drop fields require actual Destruct"))?;
        let application = self
            .facts
            .types
            .intern_trait_application(Application {
                item: ItemId(role.definition.0),
                arguments: GenericArgs::EMPTY,
            })
            .map_err(|e| self.error(key, e))?;
        let environment = self
            .facts
            .generics
            .environments
            .get(&contract.owner)
            .cloned();
        if self.facts.generics.caller_scope_pending(contract.owner)
            || (environment.is_none()
                && (self
                    .facts
                    .generics
                    .parameters_in_scope(contract.owner)
                    .next()
                    .is_some()
                    || self
                        .facts
                        .generics
                        .predicates_in_scope(contract.owner)
                        .next()
                        .is_some()))
        {
            return Err(self.error(
                key,
                "const Drop fields require the original completed impl environment",
            ));
        }
        let mut proof =
            crate::sema::nominals::Proof::for_declaration(environment, contract.parameters, first);
        for ty in types {
            let requirement = Predicate {
                ty,
                bound: Bound::MaybeConstTrait(application),
            };
            if !proof
                .prove(&mut self.facts, requirement)
                .map_err(|e| self.error(key, e))?
            {
                return Err(self.error(key,"const Drop declaration field requires const Destruct in its actual impl environment"));
            }
        }
        Ok(proof.take_trait_proof())
    }
}
