//! Source coercion impls retain their typed referent proof and lifetime relation.
mod structure;
#[cfg(test)]
#[path = "../../../../../tests/invariants/refined_coercions.rs"]
mod tests;
use super::{Checker, Error, NodeKey};
use crate::{
    support::{bindings::LangItem, constness::Constness},
    types::{
        Kind, RegionId, TypeId,
        arguments::Argument,
        parameters::predicates::{Bound, Predicate},
    },
};

impl Checker<'_> {
    pub(super) fn validate_source_coercion_impl(
        &mut self,
        owner: super::DefId,
        source: NodeKey,
        predicate: Predicate,
        proof: &mut crate::sema::nominals::Proof,
    ) -> Result<Option<(RegionId, RegionId)>, Error> {
        let Some((application, Constness::Ordinary)) = predicate.bound.trait_parts() else {
            return Ok(None);
        };
        let application = self
            .facts
            .types
            .trait_application(application)
            .ok_or_else(|| self.error(source, "invalid coercion implementation predicate"))?;
        if crate::sema::compiler_items::unsizing_role(
            &self.facts,
            crate::resolve::model::DefId(application.item.0),
        )
        .map_err(|e| self.error(source, e))?
            != Some(LangItem::CoerceUnsized)
        {
            return Ok(None);
        }
        let mut arguments = self
            .facts
            .types
            .arguments(application.arguments)
            .ok_or_else(|| self.error(source, "missing coercion implementation target"))?;
        let Some(Argument::Type(target)) = arguments.next() else {
            return Err(self.error(source, "coercion implementation target must be a type"));
        };
        if arguments.next().is_some() {
            return Err(self.error(source, "coercion implementation has extra arguments"));
        }
        let (from, to, regions, role, schema) = match (
            self.facts.types.kind(predicate.ty),
            self.facts.types.kind(target),
        ) {
            (Some(Kind::Adt(..)), Some(Kind::Adt(..))) => {
                let schema = structure::schema(&mut self.facts, predicate.ty, target, proof)
                    .map_err(|e| self.error(source, e))?;
                let field = schema.fields[schema.changed as usize];
                (
                    field.source,
                    field.target,
                    None,
                    LangItem::CoerceUnsized,
                    Some(schema),
                )
            }
            // One identical pattern; the bases then relate by an ordinary
            // CoerceUnsized obligation in this impl's environment.
            (
                Some(Kind::Pattern {
                    base: from,
                    constraint: a,
                }),
                Some(Kind::Pattern {
                    base: to,
                    constraint: b,
                }),
            ) => {
                if a != b {
                    return Err(self.error(
                        source,
                        "source CoerceUnsized pattern types require one identical pattern",
                    ));
                }
                (from, to, None, LangItem::CoerceUnsized, None)
            }
            _ => {
                let (from, to, regions) = pointer_pair(&self.facts.types, predicate.ty, target)
                    .map_err(|e| self.error(source, e))?;
                (from, to, regions, LangItem::Unsize, None)
            }
        };
        self.prove_coercion_field(source, from, to, role, proof)?;
        if let Some(schema) = schema {
            self.retain_coercion_schema(owner, source, schema)?;
        }
        Ok(regions)
    }
    fn prove_coercion_field(
        &mut self,
        source: NodeKey,
        from: TypeId,
        to: TypeId,
        role: LangItem,
        proof: &mut crate::sema::nominals::Proof,
    ) -> Result<(), Error> {
        let relation = self
            .facts
            .source_bindings
            .as_deref()
            .ok_or_else(|| self.error(source, "source coercion lost its catalog"))?
            .validated_language(role)
            .map_err(|e| self.error(source, e))?
            .ok_or_else(|| {
                self.error(
                    source,
                    "source CoerceUnsized requires its actual field relation",
                )
            })?;
        let arguments = self
            .facts
            .types
            .intern_arguments(&[Argument::Type(to)])
            .map_err(|e| self.error(source, e))?;
        let application = self
            .facts
            .types
            .intern_trait_application(crate::types::traits::Application {
                item: crate::types::ItemId(relation.definition.0),
                arguments,
            })
            .map_err(|e| self.error(source, e))?;
        let before = proof.trait_selection_steps();
        let proved = proof.prove(
            &mut self.facts,
            Predicate {
                ty: from,
                bound: Bound::trait_application(application, Constness::Ordinary),
            },
        );
        self.facts.body_work.canonical_steps = self
            .facts
            .body_work
            .canonical_steps
            .saturating_add(proof.trait_selection_steps().saturating_sub(before));
        if !proved.map_err(|e| self.error(source, e))? {
            return Err(self.error(
                source,
                "source CoerceUnsized implementation does not prove its exact field relation",
            ));
        }
        Ok(())
    }
    fn retain_coercion_schema(
        &mut self,
        owner: super::DefId,
        source: NodeKey,
        schema: crate::types::unsizing::CoercionSchema,
    ) -> Result<(), Error> {
        if self.facts.unsizing_rules.is_none() {
            self.facts
                .charge_caller_inputs(1, size_of::<crate::types::unsizing::Rules>())
                .map_err(|e| self.error(source, e))?;
        }
        let pay = crate::sema::header_attributes::caller_payments(
            self.facts.must_use_contracts.as_deref(),
            true,
        )
        .map_err(|_| {
            self.error(
                source,
                "nominal coercion declaration lacks its source payer",
            )
        })?;
        self.facts
            .unsizing_rules
            .get_or_insert_default()
            .retain_nominal_schema(
                &mut self.facts.types,
                crate::types::ItemId(owner.0),
                schema,
                &|w, b| pay.charge(w, b),
            )
            .map_err(|e| Error::at(source.file, 0, e))
    }
}

type PointerPair = (TypeId, TypeId, Option<(RegionId, RegionId)>);

fn pointer_pair(
    types: &crate::types::Store,
    source: TypeId,
    target: TypeId,
) -> Result<PointerPair, &'static str> {
    match (types.kind(source), types.kind(target)) {
        (
            Some(Kind::Reference {
                referent: from,
                mutable: a,
                region: longer,
            }),
            Some(Kind::Reference {
                referent: to,
                mutable: b,
                region: shorter,
            }),
        ) if a || !b => Ok((from, to, Some((longer, shorter)))),
        (
            Some(Kind::Reference {
                referent: from,
                mutable: a,
                ..
            }),
            Some(Kind::Pointer {
                pointee: to,
                mutable: b,
            }),
        ) if a || !b => Ok((from, to, None)),
        (
            Some(Kind::Pointer {
                pointee: from,
                mutable: a,
            }),
            Some(Kind::Pointer {
                pointee: to,
                mutable: b,
            }),
        ) if a || !b => Ok((from, to, None)),
        _ => {
            Err("source CoerceUnsized implementation does not retain a permitted pointer coercion")
        }
    }
}
