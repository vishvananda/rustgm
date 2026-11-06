//! A documentary attribute cannot assert builtin `CoercePointee` validity.
use super::{Checker, DefId, Error, NodeKey};
use crate::{
    support::bindings::LangItem,
    types::{
        Kind,
        aggregate::{Kind as Aggregate, Representation},
        parameters::predicates::Predicate,
    },
};

impl Checker<'_> {
    pub(super) fn require_pointee_validation_origin(
        &self,
        source: NodeKey,
        definition: DefId,
    ) -> Result<(), Error> {
        let Some(actual) = self
            .report
            .resolution
            .source_bindings
            .as_deref()
            .and_then(|catalog| catalog.language(LangItem::CoercePointeeValidated))
        else {
            return Ok(());
        };
        if actual.definition == definition && !self.report.derived_coerce_pointee_impl(source)? {
            return Err(self.error(
                source,
                "CoercePointeeValidated implementation requires an authenticated builtin derive",
            ));
        }
        Ok(())
    }

    pub(super) fn validate_pointee_validation_target(
        &self,
        source: NodeKey,
        predicate: Predicate,
    ) -> Result<(), Error> {
        let Some(catalog) = self.facts.source_bindings.as_deref() else {
            return Ok(());
        };
        let Some(role) = catalog.language(LangItem::CoercePointeeValidated) else {
            return Ok(());
        };
        let Some((application, _)) = predicate.bound.trait_parts() else {
            return Ok(());
        };
        let actual = self
            .facts
            .types
            .trait_application(application)
            .ok_or_else(|| self.error(source, "invalid validation marker application"))?;
        if actual.item.0 != role.definition.0 {
            return Ok(());
        }
        catalog
            .validated_language(LangItem::CoercePointeeValidated)
            .map_err(|e| self.error(source, e))?;
        self.require_pointee_validation_origin(source, role.definition)?;
        crate::sema::header_attributes::charge_source_contract_query(&self.facts, 8)?;
        let Some(Kind::Adt(item, _)) = self.facts.types.kind(predicate.ty) else {
            return Err(self.error(
                source,
                "CoercePointee validity requires its actual Struct target",
            ));
        };
        let definition = self.facts.aggregates.get(item).ok_or_else(|| {
            self.error(
                source,
                "CoercePointee validity requires its completed fields",
            )
        })?;
        if definition.kind != Aggregate::Struct
            || definition.repr.representation != Representation::Transparent
            || self
                .facts
                .aggregates
                .variants(definition)
                .first()
                .and_then(|variant| self.facts.types.list(variant.fields))
                .is_none_or(<[_]>::is_empty)
        {
            return Err(self.error(
                source,
                "CoercePointee validity requires a nonempty actual transparent Struct",
            ));
        }
        Ok(())
    }
}
