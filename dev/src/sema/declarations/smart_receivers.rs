//! Receiver validity consumes actual source traits and retains their lifetime debts.
use super::{Checker, DefId, Error, NodeKey, Type, TypeId};
use crate::{
    sema::model::Function,
    support::{bindings::LangItem, hash::HashSet},
    types::{
        GenericArgs, ItemId,
        parameters::predicates::{Bound, Predicate},
        projection,
        traits::Application,
    },
};
impl Checker<'_> {
    pub(super) fn validate_smart_receiver(
        &mut self,
        function: &Function,
        key: NodeKey,
        mut current: TypeId,
        target: TypeId,
    ) -> Result<TypeId, Error> {
        let report = self.report;
        let error = |message| crate::sema::model::Declarations::error(report, key, message);
        let environment = self
            .facts
            .generics
            .environments
            .get(&function.definition)
            .cloned();
        let mut proof = self.declaration_proof(function.definition, environment);
        let mut visited = HashSet::default();
        let result = loop {
            if current == target
                || matches!((self.facts.types.kind(current),self.facts.types.kind(target)),(Some(Type::Adt(a,_)),Some(Type::Adt(b,_)))if a==b)
            {
                break current;
            }
            if visited.len() >= 1_000_000 || !visited.insert(current) {
                return Err(error(
                    "receiver Deref chain is cyclic or exceeds its work budget",
                ));
            }
            crate::sema::header_attributes::charge_source_contract_query(&self.facts, 4)?;
            let (legacy, deref, associated) = receiver_roles(&self.facts).map_err(error)?;
            require_trait(
                &mut self.facts,
                &mut proof,
                current,
                legacy,
                "receiver requires its actual LegacyReceiver implementation",
            )
            .map_err(|e| {
                crate::sema::model::Declarations::error(
                    report,
                    key,
                    format!("receiver LegacyReceiver proof failed: {e}"),
                )
            })?;
            if let Some(Type::Reference { referent, .. }) = self.facts.types.kind(current) {
                current = referent;
                continue;
            }
            let application = require_trait(
                &mut self.facts,
                &mut proof,
                current,
                deref,
                "receiver requires its actual Deref implementation",
            )
            .map_err(|e| {
                crate::sema::model::Declarations::error(
                    report,
                    key,
                    format!("receiver Deref proof failed: {e}"),
                )
            })?;
            let projection = self
                .facts
                .types
                .intern(Type::Projection(projection::Application {
                    item: ItemId(associated.0),
                    subject: current,
                    trait_application: application,
                    arguments: GenericArgs::EMPTY,
                }))
                .map_err(error)?;
            current = proof.normalize(&mut self.facts, projection).map_err(|e| {
                crate::sema::model::Declarations::error(
                    report,
                    key,
                    format!("receiver Target normalization failed: {e}"),
                )
            })?;
        };
        self.retain_trait_proof(function.definition, key, &mut proof);
        Ok(result)
    }
}
fn receiver_roles(
    facts: &crate::sema::model::Declarations,
) -> Result<(DefId, DefId, DefId), &'static str> {
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("smart receiver requires actual source declarations")?;
    let legacy = catalog
        .validated_language(LangItem::LegacyReceiver)?
        .ok_or("receiver lacks its completed actual LegacyReceiver trait")?;
    let deref = catalog
        .validated_language(LangItem::Deref)?
        .ok_or("receiver lacks its completed actual Deref trait")?;
    let target = catalog
        .validated_language(LangItem::DerefTarget)?
        .ok_or("receiver lacks its completed actual Deref Target")?;
    if target.parent != Some(deref.definition) {
        return Err("receiver Deref Target belongs to another actual trait");
    }
    Ok((legacy.definition, deref.definition, target.definition))
}

fn require_trait(
    facts: &mut crate::sema::model::Declarations,
    proof: &mut crate::sema::nominals::Proof,
    subject: TypeId,
    definition: DefId,
    message: &'static str,
) -> Result<crate::types::traits::ApplicationId, &'static str> {
    let application = facts.types.intern_trait_application(Application {
        item: ItemId(definition.0),
        arguments: GenericArgs::EMPTY,
    })?;
    if !proof.prove(
        facts,
        Predicate {
            ty: subject,
            bound: Bound::Trait(application),
        },
    )? {
        return Err(message);
    }
    Ok(application)
}
