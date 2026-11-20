//! Source metadata declaration authority is independent of pointer representation.
mod automatic;
mod representation;
pub(in crate::sema) use representation::{Plan as MetadataPlan, Step as MetadataStep};
mod published_family;
mod schemas;
mod sealed_declarations;
pub(in crate::sema) use automatic::{Readiness, Selection, select};
#[cfg(test)]
#[path = "../../../tests/invariants/pointee_consumers.rs"]
mod consumer_tests;
#[cfg(test)]
#[path = "../../../tests/invariants/pointee_contracts.rs"]
mod tests;
use super::{Catalog, Declaration, Error, LangItem, Report, Role};
use crate::{
    resolve::body_origin::DeclarationOrigin,
    sema::{crates, header_attributes, model::Declarations},
    types::projection,
};
pub(super) use sealed_declarations::Contracts;
use std::rc::Rc;

/// Its constructor binds a complete use to the actual current source catalog.
/// This declaration token grants neither a trait proof nor normalized equality.
pub(in crate::sema) struct CheckedMember {
    contract: Rc<Contracts>,
    application: projection::Application,
}
impl CheckedMember {
    pub(in crate::sema) fn current_facts(
        &self,
        facts: &Declarations,
    ) -> Result<projection::Application, Error> {
        require_owner(facts, &self.contract)?;
        self.contract.current_facts(facts)?;
        self.contract.application(facts, self.application)?;
        Ok(self.application)
    }
    pub(in crate::sema) fn current(
        &self,
        report: &Report,
        facts: &Declarations,
    ) -> Result<projection::Application, Error> {
        require_owner(facts, &self.contract)?;
        self.contract.current(report, facts)?;
        self.contract.application(facts, self.application)?;
        Ok(self.application)
    }
}
fn require_owner(facts: &Declarations, expected: &Rc<Contracts>) -> Result<(), Error> {
    charge(facts, expected.plan.trait_, 1)?;
    let actual = facts
        .source_bindings
        .as_deref()
        .and_then(|c| c.traits.as_deref())
        .and_then(|v| v.pointee.as_ref())
        .ok_or_else(|| Error::unlocated("source Metadata use lost its actual role owner"))?;
    if !Rc::ptr_eq(actual, expected) {
        return Err(Error::unlocated(
            "source Metadata use belongs to another role catalog",
        ));
    }
    expected.completed_roles(facts)
}

/// Facts-only solver replay cannot inspect a separately changed Report. Real
/// body and publication boundaries reauthenticate the source they actually use.
pub(in crate::sema) fn authenticate(report: &Report, facts: &Declarations) -> Result<(), Error> {
    if report.package.staged_consumer().is_some() {
        return Ok(());
    }
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(());
    };
    if catalog
        .role(LangItem::PointeeTrait)
        .map_err(Error::unlocated)?
        .is_none()
    {
        return Ok(());
    }
    catalog
        .require_provider_report(report)
        .map_err(Error::unlocated)?;
    let contract = catalog
        .traits
        .as_deref()
        .and_then(|roles| roles.pointee.as_ref())
        .ok_or_else(|| Error::unlocated("source Pointee declaration seal is not complete"))?;
    contract.completed_roles(facts)?;
    contract.current(report, facts)
}

impl Catalog {
    pub(in crate::sema) fn pointee_member(
        &self,
        report: &Report,
        facts: &Declarations,
        application: projection::Application,
    ) -> Result<CheckedMember, Error> {
        self.require_provider_report(report)
            .map_err(Error::unlocated)?;
        let contract = self
            .traits
            .as_deref()
            .and_then(|v| v.pointee.as_ref())
            .ok_or_else(|| {
                Error::unlocated("source Metadata declaration contracts are not complete")
            })?;
        charge(facts, contract.plan.trait_, 1)?;
        if !facts
            .source_bindings
            .as_deref()
            .is_some_and(|actual| std::ptr::eq(actual, self))
        {
            return Err(Error::unlocated(
                "source Metadata query uses another actual catalog",
            ));
        }
        contract.completed_roles(facts)?;
        contract.current(report, facts)?;
        contract.application(facts, application)?;
        Ok(CheckedMember {
            contract: Rc::clone(contract),
            application,
        })
    }
}

fn charge(facts: &Declarations, role: Role, steps: usize) -> Result<(), Error> {
    match role.origin {
        DeclarationOrigin::Source(_) => {
            header_attributes::charge_source_contract_query(facts, steps)
        }
        DeclarationOrigin::Imported(_) => crates::charge_source_header_query(facts, steps),
    }
}

fn reserve(facts: &mut Declarations, role: Role, bytes: usize) -> Result<(), Error> {
    match role.origin {
        DeclarationOrigin::Source(_) => {
            header_attributes::reserve_source_contracts(facts, 1, bytes)
        }
        DeclarationOrigin::Imported(_) => crates::charge_source_headers(facts, 1, bytes),
    }
}

pub(in crate::sema) fn retain(
    facts: &mut Declarations,
    role: Role,
    steps: usize,
    bytes: usize,
) -> Result<(), Error> {
    match role.origin {
        DeclarationOrigin::Source(_) => {
            header_attributes::charge_source_retention(facts, steps, bytes)
        }
        DeclarationOrigin::Imported(_) => crates::charge_source_headers(facts, steps, bytes),
    }
}

pub(super) fn prepare_imported(
    report: &Report,
    facts: &mut Declarations,
    roles: &mut super::roles::RoleSet,
) -> Result<Option<Rc<Contracts>>, Error> {
    let mut steps = 0;
    let checked = published_family::plan(report, facts, &mut steps);
    crates::charge_source_headers(facts, steps, 0)?;
    let Some(plan) = checked.map_err(Error::unlocated)? else {
        return Ok(None);
    };
    let (steps, bytes) = Contracts::payment(facts, plan)?;
    crates::charge_source_headers(facts, steps, bytes)?;
    let arena = facts.types.arena_identity();
    let contract = Contracts::capture(report, facts, plan, arena)?;
    roles.insert(LangItem::PointeeTrait);
    roles.insert(LangItem::MetadataType);
    if plan.nominal.is_some() {
        roles.insert(LangItem::DynMetadata);
    }
    Ok(Some(Rc::new(contract)))
}

/// Called only after real associated results and nominal declarations complete.
pub(in crate::sema) fn prepare(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let has_role = facts.source_bindings.as_deref().is_some_and(|catalog| {
        [
            LangItem::PointeeTrait,
            LangItem::MetadataType,
            LangItem::DynMetadata,
        ]
        .into_iter()
        .any(|role| catalog.language(role).is_some())
    });
    if !has_role {
        return Ok(());
    }
    if facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| catalog.traits.as_deref())
        .is_some_and(|roles| roles.pointee.is_some())
    {
        return authenticate(report, facts);
    }
    header_attributes::reserve_source_contracts(facts, 3, 0)?;
    let plan = schemas::plan(report, facts)?
        .ok_or_else(|| Error::unlocated("source Pointee role disappeared during completion"))?;
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or_else(|| Error::unlocated("source Pointee catalog disappeared"))?;
    if catalog
        .traits
        .as_deref()
        .is_some_and(|v| v.pointee.is_some())
    {
        return Err(Error::unlocated(
            "source Pointee contracts were completed twice",
        ));
    }
    let needs_ledger = catalog.traits.is_none();
    let (steps, bytes) = Contracts::payment(facts, plan)?;
    header_attributes::reserve_source_contracts(
        facts,
        steps,
        bytes.saturating_add(usize::from(needs_ledger) * size_of::<super::roles::Validated>()),
    )?;
    let arena = facts.types.arena_identity();
    let contract = Rc::new(Contracts::capture(report, facts, plan, arena)?);
    contract.current(report, facts)?;
    let catalog = facts
        .source_bindings
        .as_deref_mut()
        .ok_or_else(|| Error::unlocated("source Pointee catalog disappeared"))?;
    let ledger = catalog.traits.get_or_insert_with(|| {
        Box::new(super::roles::Validated {
            roles: super::roles::RoleSet::EMPTY,
            methods: [None; 3],
            pointee: None,
            discriminant: None,
            range: None,
            field: None,
        })
    });
    ledger.roles.insert(LangItem::PointeeTrait);
    ledger.roles.insert(LangItem::MetadataType);
    if plan.nominal.is_some() {
        ledger.roles.insert(LangItem::DynMetadata);
    }
    ledger.pointee = Some(contract);
    Ok(())
}

/// Readiness only wakes an existing continuation; every use reauthenticates.
pub(in crate::sema) fn ready(facts: &Declarations, member: crate::resolve::model::DefId) -> bool {
    facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| catalog.traits.as_deref())
        .and_then(|roles| roles.pointee.as_ref())
        .is_some_and(|contract| contract.plan.member.definition == member)
}

/// Only a completed, current source seal can identify an early prepared member.
/// A missing contract takes the ordinary producer path rather than implying
/// checked absence or a completed associated result.
pub(in crate::sema) fn prepared_member(
    report: &Report,
    facts: &Declarations,
    definition: crate::resolve::model::DefId,
) -> Result<bool, Error> {
    let Some(contract) = facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| catalog.traits.as_deref())
        .and_then(|roles| roles.pointee.as_ref())
        .filter(|contract| contract.plan.member.definition == definition)
    else {
        return Ok(false);
    };
    require_owner(facts, contract)?;
    contract.current(report, facts)?;
    Ok(true)
}

/// Authenticate the actual completed member before publication.
pub(in crate::sema) fn require_transport(
    report: &Report,
    facts: &Declarations,
    enabled: bool,
) -> Result<(), Error> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(());
    };
    let Some(declaration) = catalog.language(LangItem::MetadataType) else {
        if [LangItem::PointeeTrait, LangItem::DynMetadata]
            .into_iter()
            .any(|role| catalog.language(role).is_some())
        {
            return Err(Error::unlocated(
                "source Pointee transport lacks its actual Metadata declaration",
            ));
        }
        return Ok(());
    };
    let contract = facts
        .associated
        .require_projection_header(declaration.definition)
        .map_err(|message| Declarations::error(report, declaration.origin, message))?;
    let Some(crate::types::Kind::Projection(application)) = facts.types.kind(contract.projection)
    else {
        return Err(Declarations::error(
            report,
            declaration.origin,
            "source Metadata transport has no actual canonical projection",
        ));
    };
    let checked = catalog.pointee_member(report, facts, application)?;
    checked.current(report, facts)?;
    if enabled {
        return Ok(());
    }
    Err(Declarations::error(
        report,
        declaration.origin,
        "source Pointee contract transport is not implemented",
    ))
}
