//! Arbitrary Self reachability is type proof, separate from executable dereference.
use crate::{
    resolve::{
        bindings::FeatureName, expansion::Report, model::DefId, package::Error, unit::NodeKey,
    },
    sema::{model::Declarations, nominals::Proof},
    support::bindings::LangItem,
    types::{
        GenericArgs, ItemId, Kind, TypeId,
        parameters::predicates::{Bound, Predicate},
        projection,
        traits::Application,
    },
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::sema) enum Level {
    Legacy,
    Basic,
    Pointers,
}

pub(in crate::sema) fn level(report: &Report, key: NodeKey) -> Result<Level, Error> {
    let mut allowances = report.macro_allowances();
    let span = report
        .package
        .unit(key.file)
        .configured
        .ast
        .node(key.node)
        .span;
    for (pointers, level, name) in [
        (true, Level::Pointers, "arbitrary_self_types_pointers"),
        (false, Level::Basic, "arbitrary_self_types"),
    ] {
        if report
            .package
            .requested_arbitrary_receivers(pointers)
            .map_err(Error::unlocated)?
            || allowances.allows_at(
                key.file,
                span,
                &FeatureName::new(name).map_err(Error::unlocated)?,
            )?
        {
            return Ok(level);
        }
    }
    Ok(Level::Legacy)
}

/// A successful projection carries ordinary implementation and lifetime debts
/// in the caller-owned proof. It grants no load, reference or adjustment.
pub(in crate::sema) fn next(
    report: &Report,
    facts: &mut Declarations,
    proof: &mut Proof,
    current: TypeId,
    level: Level,
) -> Result<Option<TypeId>, &'static str> {
    crate::sema::header_attributes::charge_source_contract_query(facts, 4)
        .map_err(|_| "receiver source work budget exceeded")?;
    if level == Level::Legacy {
        let catalog = facts
            .source_bindings
            .as_deref()
            .ok_or("receiver lacks its actual source catalog")?;
        let legacy = catalog
            .validated_language(LangItem::LegacyReceiver)?
            .ok_or("receiver lacks its completed actual LegacyReceiver trait")?
            .definition;
        if require_trait(facts, proof, current, legacy)?.is_none() {
            return Ok(None);
        }
    }
    match facts.types.kind(current) {
        Some(Kind::Reference { referent, .. }) => return Ok(Some(referent)),
        Some(Kind::Pointer { pointee, .. }) if level == Level::Pointers => {
            return Ok(Some(pointee));
        }
        _ => {}
    }
    let (role, member) = if level == Level::Legacy {
        (LangItem::Deref, LangItem::DerefTarget)
    } else {
        (LangItem::Receiver, LangItem::ReceiverTarget)
    };
    if role == LangItem::Receiver
        && facts
            .source_bindings
            .as_deref()
            .is_none_or(|catalog| catalog.language(role).is_none())
    {
        return Ok(None);
    }
    let (definition, associated) = roles(report, facts, role, member)?;
    let Some(application) = require_trait(facts, proof, current, definition)? else {
        return Ok(None);
    };
    let projected = facts
        .types
        .intern(Kind::Projection(projection::Application {
            item: ItemId(associated.0),
            subject: current,
            trait_application: application,
            arguments: GenericArgs::EMPTY,
        }))?;
    proof.normalize(facts, projected).map(Some)
}

fn roles(
    report: &Report,
    facts: &Declarations,
    role: LangItem,
    member: LangItem,
) -> Result<(DefId, DefId), &'static str> {
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("receiver lacks its actual source catalog")?;
    catalog.require_provider_report(report)?;
    let declaration = catalog
        .validated_language(role)?
        .ok_or("receiver lacks its completed actual trait")?;
    let associated = catalog
        .validated_language(member)?
        .ok_or("receiver lacks its completed actual Target")?;
    if associated.parent != Some(declaration.definition) {
        return Err("receiver Target belongs to another actual trait");
    }
    let header = super::headers::Trait::source(report, facts, declaration)?;
    header.canonical_inputs(facts)?;
    if !header.parameters.is_empty() {
        return Err("receiver trait application requires its actual generic arguments");
    }
    Ok((declaration.definition, associated.definition))
}

fn require_trait(
    facts: &mut Declarations,
    proof: &mut Proof,
    subject: TypeId,
    definition: DefId,
) -> Result<Option<crate::types::traits::ApplicationId>, &'static str> {
    let application = facts.types.intern_trait_application(Application {
        item: ItemId(definition.0),
        arguments: GenericArgs::EMPTY,
    })?;
    Ok(proof
        .prove(
            facts,
            Predicate {
                ty: subject,
                bound: Bound::Trait(application),
            },
        )?
        .then_some(application))
}
