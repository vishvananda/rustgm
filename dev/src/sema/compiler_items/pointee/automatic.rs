//! Universal Pointee evidence retains its existing source owner, not a result.
use super::{CheckedMember, Contracts, Declarations, Error, LangItem};
use crate::{
    resolve::{body_origin::DeclarationOrigin, model::DefId},
    support::constness::Constness,
    types::{parameters::predicates::Predicate, projection},
};
use std::rc::Rc;

/// The actual proof/query separately retains its complete requested application.
#[derive(Clone)]
pub(in crate::sema) struct Selection(Rc<Contracts>);

pub(in crate::sema) enum Readiness {
    Unrelated,
    Pending(DefId),
    Ready(Selection),
}

pub(in crate::sema) fn select(
    facts: &mut Declarations,
    goal: Predicate,
) -> Result<Readiness, Error> {
    let Some((application, effect)) = goal.bound.trait_parts() else {
        return Ok(Readiness::Unrelated);
    };
    let application = facts
        .types
        .trait_application(application)
        .ok_or_else(|| Error::unlocated("source Pointee goal has invalid trait inputs"))?;
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(Readiness::Unrelated);
    };
    let Some(role) = catalog
        .role(LangItem::PointeeTrait)
        .map_err(Error::unlocated)?
    else {
        return Ok(Readiness::Unrelated);
    };
    if application.item.0 != role.definition.0 {
        return Ok(Readiness::Unrelated);
    }
    if matches!(role.origin, DeclarationOrigin::Imported(_))
        && facts
            .associated
            .imported_owner()
            .is_some_and(crate::sema::crates::ImportedOwners::selected)
    {
        return Err(Error::unlocated(
            "imported Pointee selection contracts are not implemented",
        ));
    }
    super::charge(facts, role, 1)?;
    if effect != Constness::Ordinary
        || facts
            .types
            .arguments(application.arguments)
            .is_none_or(|a| a.len() != 0)
    {
        return Err(Error::unlocated(
            "source Pointee requires its complete ordinary application",
        ));
    }
    let member = catalog
        .role(LangItem::MetadataType)
        .map_err(Error::unlocated)?
        .ok_or_else(|| Error::unlocated("source Pointee lost its actual Metadata member"))?;
    let Some(contract) = catalog.traits.as_deref().and_then(|v| v.pointee.as_ref()) else {
        return Ok(Readiness::Pending(member.definition));
    };
    contract.current_facts(facts)?;
    super::reserve(facts, role, size_of::<Selection>())?;
    let contract = facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| catalog.traits.as_deref())
        .and_then(|roles| roles.pointee.as_ref())
        .ok_or_else(|| Error::unlocated("source Pointee owner disappeared after payment"))?;
    // Clone only an already retained owner, after payment. No per-query Rc
    // allocation is made, and payment cannot replace the source catalog.
    Ok(Readiness::Ready(Selection(Rc::clone(contract))))
}

impl Selection {
    pub(in crate::sema) fn retain(
        &self,
        facts: &mut Declarations,
        steps: usize,
        bytes: usize,
    ) -> Result<(), String> {
        super::retain(facts, self.0.plan.trait_, steps, bytes).map_err(|error| error.to_string())
    }

    pub(in crate::sema) fn current(&self, facts: &Declarations) -> Result<(), Error> {
        super::require_owner(facts, &self.0)?;
        self.0.current_facts(facts)
    }

    pub(in crate::sema) fn member(
        &self,
        facts: &mut Declarations,
        application: projection::Application,
    ) -> Result<CheckedMember, Error> {
        self.current(facts)?;
        self.0.application(facts, application)?;
        super::reserve(facts, self.0.plan.trait_, size_of::<CheckedMember>())?;
        Ok(CheckedMember {
            contract: Rc::clone(&self.0),
            application,
        })
    }
}
