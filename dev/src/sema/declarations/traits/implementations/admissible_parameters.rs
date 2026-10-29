//! `ConstParamTy` impls prove their actual children; nominal marker syntax is no proof.
use super::{Checker, DefId, Error, NodeKey};
use crate::{
    sema::nominals::Proof,
    support::bindings::LangItem,
    types::{
        Kind, Primitive, TypeId,
        parameters::{
            clauses::Clause,
            predicates::{Bound, Predicate},
        },
    },
};

impl Checker<'_> {
    pub(super) fn validate_source_constant_type_impl(
        &mut self,
        owner: DefId,
        source: NodeKey,
        predicate: Predicate,
        proof: &mut Proof,
    ) -> Result<(), Error> {
        let Some(catalog) = self.facts.source_bindings.as_deref() else {
            return Ok(());
        };
        let derived = self.report.derived_const_parameter_impl(source)?;
        let Some(declaration) = catalog.language(LangItem::ConstParamTy) else {
            return if derived {
                Err(self.error(
                    source,
                    "source ConstParamTy derive requires its actual language trait",
                ))
            } else {
                Ok(())
            };
        };
        let Some((application, _)) = predicate.bound.trait_parts() else {
            return Ok(());
        };
        let actual = self
            .facts
            .types
            .trait_application(application)
            .ok_or_else(|| self.error(source, "invalid ConstParamTy implementation application"))?;
        if actual.item.0 != declaration.definition.0 {
            return if derived {
                Err(self.error(
                    source,
                    "source ConstParamTy derive resolved to a different actual trait",
                ))
            } else {
                Ok(())
            };
        }
        catalog
            .validated_language(LangItem::ConstParamTy)
            .map_err(|e| self.error(source, e))?;
        crate::sema::header_attributes::reserve_source_contracts(
            &mut self.facts,
            8,
            4 * size_of::<TypeId>(),
        )?;
        let (children, needs_feature) = self.constant_children(source, predicate.ty)?;
        if needs_feature {
            self.constant_unsized_premise(owner, source, proof)?;
        }
        crate::sema::header_attributes::reserve_source_contracts(
            &mut self.facts,
            children.len(),
            0,
        )?;
        for ty in children {
            if !proof
                .prove(
                    &mut self.facts,
                    Predicate {
                        ty,
                        bound: Bound::Trait(application),
                    },
                )
                .map_err(|e| self.error(source, e))?
            {
                return Err(self.error(source,
                    "ConstParamTy implementation requires every real child to implement actual ConstParamTy"));
            }
        }
        Ok(())
    }

    fn constant_children(
        &mut self,
        source: NodeKey,
        ty: TypeId,
    ) -> Result<(Vec<TypeId>, bool), Error> {
        let mut needs_feature = false;
        let children = match self.facts.types.kind(ty) {
            Some(Kind::Primitive(Primitive::Str)) => {
                needs_feature = true;
                let producer = self.report.lint_producer().ok_or_else(|| {
                    self.error(
                        source,
                        "ConstParamTy str rule requires its actual source payer",
                    )
                })?;
                let pay = |steps, bytes| {
                    producer
                        .diagnostic_payload(source, steps, bytes)
                        .map_err(|error| error.message)
                };
                vec![
                    self.facts
                        .types
                        .intern_observed::<true>(
                            Kind::Slice(TypeId::primitive(Primitive::U8)),
                            &pay,
                        )
                        .map_err(|e| self.error(source, e))?,
                ]
            }
            Some(Kind::Primitive(kind)) if !kind.is_float() => Vec::new(),
            Some(Kind::Array { element, .. } | Kind::ArrayTerm { element, .. }) => vec![element],
            Some(Kind::Slice(element)) => {
                needs_feature = true;
                vec![element]
            }
            Some(Kind::Reference {
                mutable: false,
                referent,
                ..
            }) => {
                needs_feature = true;
                vec![referent]
            }
            Some(Kind::Tuple(list)) => {
                let length = self
                    .facts
                    .types
                    .list(list)
                    .ok_or_else(|| self.error(source, "invalid ConstParamTy tuple"))?
                    .len();
                crate::sema::header_attributes::reserve_source_contracts(
                    &mut self.facts,
                    length,
                    length.saturating_mul(4 * size_of::<TypeId>()),
                )?;
                self.facts
                    .types
                    .list(list)
                    .ok_or_else(|| self.error(source, "ConstParamTy tuple disappeared"))?
                    .to_vec()
            }
            Some(Kind::Adt(item, arguments)) => {
                if self
                    .facts
                    .types
                    .arguments(arguments)
                    .is_none_or(|args| args.len() != 0)
                {
                    return Err(self.error(
                        source,
                        "generic ConstParamTy nominal field proof requires a wider profile",
                    ));
                }
                self.constant_nominal_leaf(source, item)?;
                Vec::new()
            }
            _ => return Err(self.error(source, "this type cannot implement actual ConstParamTy")),
        };
        Ok((children, needs_feature))
    }

    fn constant_nominal_leaf(
        &mut self,
        source: NodeKey,
        item: crate::types::ItemId,
    ) -> Result<(), Error> {
        let definition =
            self.facts.aggregates.get(item).copied().ok_or_else(|| {
                self.error(source, "ConstParamTy nominal has no actual declaration")
            })?;
        if !matches!(
            definition.kind,
            crate::types::aggregate::Kind::Struct | crate::types::aggregate::Kind::Enum
        ) {
            return Err(self.error(source, "ConstParamTy requires an actual struct or enum"));
        }
        let count = self.facts.aggregates.variants(&definition).len();
        crate::sema::header_attributes::reserve_source_contracts(&mut self.facts, count, 0)?;
        for variant in self.facts.aggregates.variants(&definition) {
            if self
                .facts
                .types
                .list(variant.fields)
                .is_none_or(|fields| !fields.is_empty())
            {
                return Err(self.error(
                    source,
                    "ConstParamTy ADT payloads require their wider recursive field proof",
                ));
            }
        }
        if self.facts.members.variants(item).len() != count {
            return Err(self.error(
                source,
                "ConstParamTy nominal lost its actual member coverage",
            ));
        }
        for ordinal in 0..count {
            let id = self.facts.members.variants(item)[ordinal];
            let variant = self
                .facts
                .members
                .variants
                .get(id.0 as usize)
                .ok_or_else(|| self.error(source, "ConstParamTy variant is missing"))?;
            if variant.owner != item {
                return Err(self.error(source, "ConstParamTy variant has a foreign owner"));
            }
            let key = variant.origin.source().ok_or_else(|| {
                self.error(
                    source,
                    "ConstParamTy foreign variant contract transport requires a wider profile",
                )
            })?;
            if self
                .facts
                .non_exhaustive_contract(self.report, key)?
                .is_some()
            {
                return Err(self.error(source, "non-exhaustive ConstParamTy field lists require their actual ADT feature consumer"));
            }
        }
        Ok(())
    }

    fn constant_unsized_premise(
        &mut self,
        owner: DefId,
        source: NodeKey,
        proof: &mut Proof,
    ) -> Result<(), Error> {
        let mut selected = None;
        for clause in self.facts.generics.declared_predicates_in_scope(owner) {
            self.facts.body_work.canonical_steps =
                self.facts.body_work.canonical_steps.saturating_add(24);
            crate::sema::header_attributes::charge_source_contract_query(&self.facts, 24)?;
            if let Clause::UnstableFeature(id) = clause.value
                && self.facts.types.feature_name(id) == Some("unsized_const_params")
            {
                selected = Some(clause.value);
                break;
            }
        }
        let Some(requirement) = selected else {
            return Err(self.error(source, "ConstParamTy unsized/reference impl requires its actual unsized_const_params premise"));
        };
        if !proof
            .prove_clause(&mut self.facts, requirement)
            .map_err(|e| self.error(source, e))?
        {
            return Err(self.error(
                source,
                "ConstParamTy unsized premise is not proved in its actual impl scope",
            ));
        }
        Ok(())
    }
}
