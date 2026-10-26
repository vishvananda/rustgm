//! Source metadata declaration authority is independent of pointer representation.
mod schemas;
mod sealed_declarations;
#[cfg(test)]
#[path = "../../../tests/invariants/pointee_contracts.rs"]
mod tests;
use super::{Catalog, Declaration, Error, LangItem, Report};
use crate::{
    sema::{header_attributes, model::Declarations},
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
    pub(in crate::sema) fn current(
        &self,
        report: &Report,
        facts: &Declarations,
    ) -> Result<projection::Application, Error> {
        header_attributes::charge_source_contract_query(facts, 1)?;
        let actual = facts
            .source_bindings
            .as_deref()
            .and_then(|c| c.traits.as_deref())
            .and_then(|v| v.pointee.as_ref())
            .ok_or_else(|| Error::unlocated("source Metadata use lost its actual role owner"))?;
        if !Rc::ptr_eq(actual, &self.contract) {
            return Err(Error::unlocated(
                "source Metadata use belongs to another role catalog",
            ));
        }
        self.contract.completed_roles(facts)?;
        self.contract.current(report, facts)?;
        self.contract.application(facts, self.application)?;
        Ok(self.application)
    }
}
impl Catalog {
    pub(in crate::sema) fn pointee_member(
        &self,
        report: &Report,
        facts: &Declarations,
        application: projection::Application,
    ) -> Result<CheckedMember, Error> {
        header_attributes::charge_source_contract_query(facts, 1)?;
        self.require_provider_report(report)
            .map_err(Error::unlocated)?;
        let contract = self
            .traits
            .as_deref()
            .and_then(|v| v.pointee.as_ref())
            .ok_or_else(|| {
                Error::unlocated("source Metadata declaration contracts are not complete")
            })?;
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
            roles: 0,
            methods: [None; 3],
            pointee: None,
        })
    });
    ledger.roles |= (1 << LangItem::PointeeTrait.index()) | (1 << LangItem::MetadataType.index());
    if plan.nominal.is_some() {
        ledger.roles |= 1 << LangItem::DynMetadata.index();
    }
    ledger.pointee = Some(contract);
    Ok(())
}

/// No existing artifact profile transports these new declaration contracts.
/// Authenticate the actual completed member before refusing publication; local
/// source analysis remains available and no wire row is silently dropped.
pub(in crate::sema) fn require_transport(
    report: &Report,
    facts: &Declarations,
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
    Err(Declarations::error(
        report,
        declaration.origin,
        "source Pointee contract transport is not implemented",
    ))
}
