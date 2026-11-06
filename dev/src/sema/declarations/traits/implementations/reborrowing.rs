//! Actual reborrowing impls prove their fields in the existing owner environment.
mod relations;
pub(super) mod structure;
use super::{Checker, DefId, Error, NodeKey};
use crate::{
    sema::{model::Declarations, nominals::Proof},
    support::{bindings::LangItem, constness::Constness},
    types::{arguments::Argument, parameters::predicates::Predicate},
};

impl Checker<'_> {
    pub(super) fn validate_source_reborrowing_impl(
        &mut self,
        owner: DefId,
        source: NodeKey,
        predicate: Predicate,
        proof: &mut Proof,
    ) -> Result<(), Error> {
        validate(&mut self.facts, owner, predicate, proof)
            .map_err(|message| self.error(source, message))
    }
}

fn validate(
    facts: &mut Declarations,
    owner: DefId,
    predicate: Predicate,
    proof: &mut Proof,
) -> Result<(), &'static str> {
    let Some((id, Constness::Ordinary)) = predicate.bound.trait_parts() else {
        return Ok(());
    };
    let application = facts
        .types
        .trait_application(id)
        .ok_or("reborrowing impl has an invalid actual trait application")?;
    let Some(role) = role(facts, application.item)? else {
        return Ok(());
    };
    // The actual owner binder includes anonymous impl lifetimes. A lifetime-only
    // impl need not have an entry in the type/const parameter argument table.
    let schema = facts
        .associated
        .schema(owner)
        .ok_or("reborrowing implementation has no prepared owner binder")?;
    if schema.self_type != predicate.ty {
        return Err("reborrowing implementation has a different prepared Self application");
    }
    if schema.captures != 1 {
        return Err("reborrowing implementation requires exactly one impl-owned lifetime");
    }
    let target = {
        let mut arguments = facts
            .types
            .arguments(application.arguments)
            .ok_or("reborrowing impl has no actual argument list")?;
        match (role, arguments.next(), arguments.next()) {
            (LangItem::Reborrow, None, None) => None,
            (LangItem::CoerceShared, Some(Argument::Type(target)), None) => Some(target),
            _ => return Err("reborrowing impl has a different actual target argument profile"),
        }
    };
    let source = structure::view(facts, predicate.ty)?;
    if let Some(target) = target {
        let target = proof.normalize(facts, target)?;
        let target = structure::view(facts, target)?;
        if source.fields.len() > 1 || source.fields.len() != target.fields.len() {
            return Err("CoerceShared structs require equal counts of zero or one data field");
        }
        if let ([from], [to]) = (source.fields.as_slice(), target.fields.as_slice()) {
            relations::shared(facts, from, to, proof)?;
        }
    } else {
        for field in &source.fields {
            relations::reborrow(facts, field, proof)?;
        }
    }
    Ok(())
}

fn role(
    facts: &Declarations,
    actual: crate::types::ItemId,
) -> Result<Option<LangItem>, &'static str> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(None);
    };
    for role in [LangItem::Reborrow, LangItem::CoerceShared] {
        let Some(declaration) = catalog.language(role) else {
            continue;
        };
        if declaration.definition.0 != actual.0 {
            continue;
        }
        if catalog.validated_language(role)? != Some(declaration) {
            return Err("reborrowing impl requires its actual completed source trait header");
        }
        return Ok(Some(role));
    }
    Ok(None)
}

pub(super) fn declaration(
    facts: &Declarations,
    role: LangItem,
) -> Result<crate::types::ItemId, &'static str> {
    facts
        .source_bindings
        .as_deref()
        .ok_or("reborrowing field proof lost its actual source catalog")?
        .validated_language(role)?
        .map(|declaration| crate::types::ItemId(declaration.definition.0))
        .ok_or("reborrowing field proof requires its actual completed language trait")
}
