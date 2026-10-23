//! Source coercion impls retain their typed referent proof and lifetime relation.
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
        let (from, to, regions) = pointer_pair(&self.facts.types, predicate.ty, target)
            .map_err(|e| self.error(source, e))?;
        let unsize = self
            .facts
            .source_bindings
            .as_deref()
            .ok_or_else(|| self.error(source, "source coercion lost its catalog"))?
            .validated_language(LangItem::Unsize)
            .map_err(|e| self.error(source, e))?
            .ok_or_else(|| self.error(source, "source CoerceUnsized requires actual Unsize"))?;
        let arguments = self
            .facts
            .types
            .intern_arguments(&[Argument::Type(to)])
            .map_err(|e| self.error(source, e))?;
        let application = self
            .facts
            .types
            .intern_trait_application(crate::types::traits::Application {
                item: crate::types::ItemId(unsize.definition.0),
                arguments,
            })
            .map_err(|e| self.error(source, e))?;
        if !proof
            .prove(
                &mut self.facts,
                Predicate {
                    ty: from,
                    bound: Bound::trait_application(application, Constness::Ordinary),
                },
            )
            .map_err(|e| self.error(source, e))?
        {
            return Err(self.error(
                source,
                "source CoerceUnsized implementation does not prove its exact Unsize relation",
            ));
        }
        Ok(regions)
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
        (Some(Kind::Adt(..)), Some(Kind::Adt(..))) => {
            Err("source CoerceUnsized nominal implementation validation is not implemented")
        }
        _ => {
            Err("source CoerceUnsized implementation does not retain a permitted pointer coercion")
        }
    }
}
