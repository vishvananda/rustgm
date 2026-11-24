//! One source protocol owner authenticates logical discriminants, never physical tags.
#[path = "logical_tags/authority.rs"]
mod authority;
#[path = "logical_tags/classification.rs"]
mod output;
use super::{Error, LangItem, Report, Role, roles};
#[path = "logical_tags/discriminant_roles.rs"]
mod imported_role;
use crate::{
    resolve::model::DefId,
    sema::{
        crates,
        header_attributes::{self, SourceImage},
        model::Declarations,
    },
    types::{TypeId, arena, projection},
};
pub(super) use imported_role::prepare as prepare_imported;
pub(in crate::sema) use output::Output;
pub(super) use output::classify;
use std::rc::Rc;

pub(in crate::sema::compiler_items) struct Contracts {
    arena: arena::Identity,
    trait_: Role,
    member: Role,
    self_type: TypeId,
    projection: projection::Application,
    images: Option<[SourceImage; 2]>,
}

/// The owning query retains the complete goal/application and its ordinary proof.
#[derive(Clone)]
pub(in crate::sema) struct Selection(Rc<Contracts>);

pub(in crate::sema) enum Readiness {
    Unrelated,
    Pending(DefId),
    Ready(Selection),
}

pub(in crate::sema) fn is_result(facts: &Declarations, item: crate::types::ItemId) -> bool {
    facts
        .source_bindings
        .as_deref()
        .and_then(|c| c.role(LangItem::DiscriminantType).ok().flatten())
        .is_some_and(|member| member.definition.0 == item.0)
}

/// The caller already owns the real parent proof; this only obtains data authority.
pub(in crate::sema) fn for_result(
    facts: &mut Declarations,
    application: projection::Application,
) -> Result<Readiness, Error> {
    if !is_result(facts, application.item) {
        return Ok(Readiness::Unrelated);
    }
    select(
        facts,
        crate::types::parameters::predicates::Predicate {
            ty: application.subject,
            bound: crate::types::parameters::predicates::Bound::trait_application(
                application.trait_application,
                crate::support::constness::Constness::Ordinary,
            ),
        },
    )
}

impl Selection {
    pub(in crate::sema) fn current(&self, facts: &Declarations) -> Result<(), Error> {
        charge(facts, 1)?;
        let current = facts
            .source_bindings
            .as_deref()
            .and_then(|c| c.traits.as_deref())
            .and_then(|r| r.discriminant.as_ref())
            .ok_or_else(|| {
                Error::unlocated("source discriminant lost its actual protocol owner")
            })?;
        let roles = facts
            .source_bindings
            .as_deref()
            .and_then(|c| c.traits.as_deref())
            .ok_or_else(|| Error::unlocated("source discriminant lost completed role state"))?;
        if !roles.roles.contains(LangItem::DiscriminantKind)
            || !roles.roles.contains(LangItem::DiscriminantType)
            || !Rc::ptr_eq(current, &self.0)
        {
            return Err(Error::unlocated(
                "source discriminant belongs to another catalog",
            ));
        }
        self.0.current_facts(facts)
    }

    pub(in crate::sema) fn output(
        &self,
        facts: &Declarations,
        application: projection::Application,
    ) -> Result<Output, Error> {
        self.current(facts)?;
        self.0.application(facts, application)?;
        output::classify(facts, application.subject)
    }
}

pub(in crate::sema) fn select(
    facts: &mut Declarations,
    goal: crate::types::parameters::predicates::Predicate,
) -> Result<Readiness, Error> {
    let Some((application, effect)) = goal.bound.trait_parts() else {
        return Ok(Readiness::Unrelated);
    };
    let application = facts
        .types
        .trait_application(application)
        .ok_or_else(|| Error::unlocated("invalid source discriminant trait inputs"))?;
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(Readiness::Unrelated);
    };
    let Some(trait_) = catalog
        .role(LangItem::DiscriminantKind)
        .map_err(Error::unlocated)?
    else {
        return Ok(Readiness::Unrelated);
    };
    if application.item.0 != trait_.definition.0 {
        return Ok(Readiness::Unrelated);
    }
    if effect != crate::support::constness::Constness::Ordinary
        || facts
            .types
            .arguments(application.arguments)
            .is_none_or(|a| a.len() != 0)
    {
        return Err(Error::unlocated(
            "source discriminant requires its complete ordinary application",
        ));
    }
    let member = catalog
        .role(LangItem::DiscriminantType)
        .map_err(Error::unlocated)?
        .ok_or_else(|| {
            Error::unlocated("source discriminant requires its actual associated type")
        })?;
    let Some(contract) = catalog
        .traits
        .as_deref()
        .and_then(|r| r.discriminant.as_ref())
    else {
        return Ok(Readiness::Pending(member.definition));
    };
    contract.current_facts(facts)?;
    super::pointee::retain(facts, trait_, 1, size_of::<Selection>())?;
    let contract = facts
        .source_bindings
        .as_deref()
        .and_then(|c| c.traits.as_deref())
        .and_then(|r| r.discriminant.as_ref())
        .ok_or_else(|| Error::unlocated("source discriminant owner disappeared after payment"))?;
    Ok(Readiness::Ready(Selection(Rc::clone(contract))))
}

pub(in crate::sema) fn prepare(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let Some(plan) = authority::plan(report, facts)? else {
        return Ok(());
    };
    if ready(facts, plan.member.definition) {
        return authenticate(report, facts);
    }
    let fresh = facts
        .source_bindings
        .as_deref()
        .is_some_and(|c| c.traits.is_none());
    let arena_bytes = if facts.types.existing_arena_identity().is_none() {
        arena::Identity::allocation_bytes()
    } else {
        0
    };
    header_attributes::charge_source_retention(
        facts,
        4,
        size_of::<Contracts>()
            + 2 * size_of::<usize>()
            + arena_bytes
            + usize::from(fresh) * size_of::<roles::Validated>(),
    )?;
    let arena = facts.types.arena_identity();
    let images = [
        header_attributes::completed_source_image(facts, report, plan.trait_.definition)?,
        header_attributes::role_source_image(facts, report, plan.member)?,
    ];
    let contract = Rc::new(Contracts {
        arena,
        trait_: plan.trait_.into(),
        member: plan.member.into(),
        self_type: plan.self_type,
        projection: plan.projection,
        images: Some(images),
    });
    contract.current_report(report, facts)?;
    let catalog = facts
        .source_bindings
        .as_deref_mut()
        .ok_or_else(|| Error::unlocated("source discriminant catalog disappeared"))?;
    let ledger = catalog.traits.get_or_insert_with(|| {
        Box::new(roles::Validated {
            roles: roles::RoleSet::EMPTY,
            methods: [None; 3],
            pointee: None,
            discriminant: None,
            range: None,
            field: None,
            comparisons: crate::sema::compiler_items::comparison_contract::Sealed::default(),
        })
    });
    if ledger.discriminant.is_some() {
        return Err(Error::unlocated(
            "source discriminant contracts completed twice",
        ));
    }
    ledger.roles.insert(LangItem::DiscriminantKind);
    ledger.roles.insert(LangItem::DiscriminantType);
    ledger.discriminant = Some(contract);
    Ok(())
}

pub(in crate::sema) fn ready(facts: &Declarations, member: DefId) -> bool {
    facts
        .source_bindings
        .as_deref()
        .and_then(|c| c.traits.as_deref())
        .and_then(|r| r.discriminant.as_ref())
        .is_some_and(|c| c.member.definition == member)
}

pub(in crate::sema) fn prepared_member(
    report: &Report,
    facts: &Declarations,
    member: DefId,
) -> Result<bool, Error> {
    if !ready(facts, member) {
        return Ok(false);
    }
    authenticate(report, facts)?;
    Ok(true)
}

pub(crate) fn authenticate(report: &Report, facts: &Declarations) -> Result<(), Error> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(());
    };
    if catalog.language(LangItem::DiscriminantKind).is_none() {
        return Ok(());
    }
    catalog
        .require_provider_report(report)
        .map_err(Error::unlocated)?;
    let contract = catalog
        .traits
        .as_deref()
        .and_then(|r| r.discriminant.as_ref())
        .ok_or_else(|| Error::unlocated("source discriminant contracts are not complete"))?;
    contract.current_report(report, facts)
}

pub(in crate::sema) fn require_transport(
    report: &Report,
    facts: &Declarations,
    permitted: bool,
) -> Result<(), Error> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(());
    };
    if catalog.language(LangItem::DiscriminantKind).is_none()
        && catalog.language(LangItem::DiscriminantType).is_none()
    {
        return Ok(());
    }
    authenticate(report, facts)?;
    if permitted {
        Ok(())
    } else {
        Err(Error::unlocated(
            "source discriminant protocol transport requires version 14",
        ))
    }
}

pub(in crate::sema) fn retain(facts: &mut Declarations, bytes: usize) -> Result<(), Error> {
    let role = facts
        .source_bindings
        .as_deref()
        .ok_or_else(|| Error::unlocated("source discriminant continuation has no actual catalog"))?
        .role(LangItem::DiscriminantKind)
        .map_err(Error::unlocated)?
        .ok_or_else(|| Error::unlocated("source discriminant continuation lost its actual role"))?;
    super::pointee::retain(facts, role, 1, bytes)
}

fn charge(facts: &Declarations, steps: usize) -> Result<(), Error> {
    if facts
        .source_bindings
        .as_deref()
        .is_some_and(super::Catalog::uses_imported_roles)
    {
        crates::charge_source_header_query(facts, steps)
    } else {
        header_attributes::charge_source_contract_query(facts, steps)
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/logical_discriminators.rs"]
mod tests;
