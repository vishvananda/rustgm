//! Missing or imported trait state is Unknown, never a local marker absence.
use super::annotation_store::{self as store, Marker, Policy};
use crate::support::{bindings::LangItem, crates::generics::source::TraitFlags};
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind},
        package::Error,
    },
    sema::model::Declarations,
    syntax::{ast::Kind, attribute::Input},
};
use std::sync::Arc;
pub(in crate::sema) fn marker(
    facts: &Declarations,
    report: &Report,
    definition: DefId,
) -> Result<bool, Error> {
    if report
        .resolution
        .definitions
        .get(definition.0 as usize)
        .is_some_and(|row| row.kind == DefKind::Trait && row.node.is_none())
        && let Some(catalog) = facts.source_bindings.as_deref()
        && let Some(role) = catalog
            .validated_role(LangItem::TrivialClone)
            .map_err(Error::unlocated)?
        && role.definition == definition
    {
        crate::sema::crates::charge_source_header_query(facts, LangItem::ALL.len())?;
        catalog
            .require_provider_report(report)
            .map_err(Error::unlocated)?;
        return Ok(catalog
            .neutral_attributes(role)
            .map_err(Error::unlocated)?
            .contains(crate::resolve::bindings::HeaderAttribute::UnsafeSpecialization));
    }
    if let Some(flags) = imported_policy(facts, definition)? {
        return Ok(flags & TraitFlags::UNSAFE_SPECIALIZATION != 0);
    }
    Ok(read(facts, report, definition)?.0.is_some())
}
pub(in crate::sema) fn applicable(
    facts: &Declarations,
    report: &Report,
    definition: DefId,
) -> Result<bool, Error> {
    if report
        .resolution
        .definitions
        .get(definition.0 as usize)
        .is_some_and(|d| d.node.is_none())
        && let Some(catalog) = facts.source_bindings.as_deref()
        && let Some(kind) = catalog.sizing_kind(definition).map_err(Error::unlocated)?
    {
        crate::sema::header_attributes::charge_source_contract_query(facts, 8)?;
        let role = catalog
            .validated_role(kind)
            .map_err(Error::unlocated)?
            .ok_or_else(|| Error::unlocated("missing actual imported sizing role"))?;
        let mut steps = 0;
        let header =
            crate::sema::compiler_items::headers::Trait::current(report, facts, role, &mut steps);
        crate::sema::header_attributes::charge_source_contract_query(
            facts,
            steps.saturating_sub(8),
        )?;
        let header = header.map_err(Error::unlocated)?;
        return Ok(header
            .attributes
            .contains(crate::resolve::bindings::HeaderAttribute::Specialization));
    }
    if let Some(flags) = imported_policy(facts, definition)? {
        return Ok(applicable_flag(flags));
    }
    let (marker, applicable) = read(facts, report, definition)?;
    Ok(applicable.is_some() && marker.is_none())
}
pub(super) fn imported_policy(
    facts: &Declarations,
    definition: DefId,
) -> Result<Option<u16>, Error> {
    use crate::sema::{associated::OwnerKind, crates::ImportedOwners};
    let Some(ImportedOwners::Ordinary(graphs)) = facts.associated.imported_owner() else {
        return Ok(None);
    };
    let Some(owner) = facts
        .associated
        .owner(definition)
        .and_then(|owner| facts.associated.get(owner))
        .copied()
        .filter(|owner| {
            owner.source.is_none()
                && matches!(owner.kind, OwnerKind::Trait { definition: id, .. } if id == definition)
        })
    else {
        return Ok(None);
    };
    crate::sema::crates::charge_source_header_query(facts, 8)?;
    let graph = graphs
        .for_definition(definition)
        .map_err(Error::unlocated)?;
    graph
        .require_store(&facts.types)
        .map_err(Error::unlocated)?;
    graph.specialization_policy(owner).map_err(Error::unlocated)
}

pub(super) const fn applicable_flag(flags: u16) -> bool {
    flags & TraitFlags::SPECIALIZATION != 0 && flags & TraitFlags::UNSAFE_SPECIALIZATION == 0
}

fn read(
    facts: &Declarations,
    report: &Report,
    definition: DefId,
) -> Result<(Option<Marker>, Option<Marker>), Error> {
    let unknown = || Error::unlocated("specialization marker is unknown or untransported");
    let actual = report
        .resolution
        .definitions
        .get(definition.0 as usize)
        .ok_or_else(unknown)?;
    let key = actual
        .node
        .filter(|_| actual.kind == DefKind::Trait)
        .ok_or_else(unknown)?;
    let table = facts
        .must_use_contracts
        .as_deref()
        .and_then(|owner| owner.specialization.as_deref())
        .filter(|table| table.complete)
        .ok_or_else(unknown)?;
    table.charge(1, 0).map_err(Error::unlocated)?;
    if table.definitions.get(&definition) != Some(&key) {
        return Err(unknown());
    }
    let row = table.sources.get(&key).ok_or_else(unknown)?;
    let unit = report
        .package
        .units
        .get(key.file.0 as usize)
        .ok_or_else(unknown)?;
    if !Arc::ptr_eq(&row.image, unit.source.original_image())
        || row.syntax != store::trait_syntax(report, key)?
    {
        return Err(Error::at(
            key.file,
            0,
            "specialization trait source incarnation changed",
        ));
    }
    for owner in &row.owners {
        table.charge(1, 0).map_err(Error::unlocated)?;
        let current = unit
            .owner_indices
            .get(owner.key.node.get() as usize)
            .copied()
            .flatten()
            .and_then(|index| unit.configured.owners.get(index.get() as usize - 1));
        if current.is_none_or(|actual| {
            !actual.enabled || actual.node != owner.key.node || actual.attributes != owner.effects
        }) || unit
            .configured
            .ast
            .nodes
            .get(owner.key.node.get() as usize)
            .is_none_or(|node| node.kind != owner.syntax)
        {
            return Err(Error::at(
                key.file,
                0,
                "specialization trait configured owner changed",
            ));
        }
        let mut node = owner.key.node;
        loop {
            table.charge(1, 0).map_err(Error::unlocated)?;
            if let Kind::Attributed { value, .. } = unit.configured.ast.node(node).kind {
                node = value;
            } else {
                break;
            }
        }
        if node != key.node {
            return Err(Error::at(
                key.file,
                0,
                "specialization marker owner no longer decorates trait",
            ));
        }
    }
    if let Some(marker) = row.marker {
        validate_marker(report, marker, Policy::Unsafe)?;
    }
    if let Some(marker) = row.applicable {
        validate_marker(report, marker, Policy::Applicable)?;
    }
    Ok((row.marker, row.applicable))
}
fn validate_marker(report: &Report, marker: Marker, policy: Policy) -> Result<(), Error> {
    let error = |message| Error::at(marker.owner.file, marker.span.start, message);
    let unit = report.package.unit(marker.owner.file);
    let owner = unit
        .owner_indices
        .get(marker.owner.node.get() as usize)
        .copied()
        .flatten()
        .and_then(|index| unit.configured.owners.get(index.get() as usize - 1));
    let (attribute, forest) = unit
        .retained_attributes(marker.effect..marker.effect + 1)
        .next()
        .ok_or_else(|| error("specialization marker retained effect disappeared"))?;
    if owner.is_none_or(|owner| !owner.enabled || !owner.attributes.contains(&marker.effect))
        || attribute.span != marker.span
        || attribute.input != Input::Empty
        || attribute.unsafe_wrapper
        || attribute.name(forest, unit.source.text()) != Some(policy.name())
        || forest
            .tree(attribute.path.start)
            .is_none_or(|tree| tree.span != marker.name)
    {
        return Err(error(
            "specialization marker actual source contract changed",
        ));
    }
    if matches!(policy, Policy::Applicable) {
        crate::resolve::bindings::root_options::require_internal(
            report,
            &mut report.macro_allowances(),
            marker.owner.file,
            attribute,
            forest,
        )?;
    } else if !crate::sema::parentheses::feature(
        report,
        &mut report.macro_allowances(),
        (marker.owner.file, marker.name),
        false,
    )? {
        return Err(error("specialization marker feature origin changed"));
    }

    Ok(())
}

impl Declarations {
    pub(in crate::sema) fn publication_specialization_policy(
        &self,
        report: &Report,
        definition: DefId,
    ) -> Result<u16, Error> {
        if self
            .must_use_contracts
            .as_deref()
            .is_none_or(|owner| owner.specialization.is_none())
        {
            return Ok(0);
        }
        let (marker, applicable) = read(self, report, definition)?;
        Ok(
            (u16::from(marker.is_some()) * TraitFlags::UNSAFE_SPECIALIZATION)
                | (u16::from(applicable.is_some()) * TraitFlags::SPECIALIZATION),
        )
    }

    pub(crate) fn publication_header_contracts<'a>(
        &'a self,
        report: &Report,
        definition: DefId,
    ) -> Result<crate::sema::header_attributes::HeaderAttributeView<'a>, Error> {
        self.profiled_publication_header_contracts(report, definition, false)
    }

    /// Publication requires completed source producers before rejecting untransported policy.
    pub(in crate::sema) fn profiled_publication_header_contracts<'a>(
        &'a self,
        report: &Report,
        definition: DefId,
        declaration_policies: bool,
    ) -> Result<crate::sema::header_attributes::HeaderAttributeView<'a>, Error> {
        use crate::sema::header_attributes::ContractKnowledge;
        let view = self.header_attribute_contracts(report, definition)?;
        let actual = report
            .resolution
            .definitions
            .get(definition.0 as usize)
            .ok_or_else(|| Error::unlocated("specialization publication header is missing"))?;
        if actual.kind != DefKind::Trait {
            return Ok(view);
        }
        if matches!(view.must_use, ContractKnowledge::Unknown)
            || matches!(view.inline, ContractKnowledge::Unknown)
        {
            return Err(Error::unlocated(
                "specialization publication requires completed source attribute producers",
            ));
        }
        if declaration_policies {
            // The typed trait header carries the completed policy.
            return Ok(view);
        }
        if self
            .must_use_contracts
            .as_deref()
            .is_some_and(|owner| owner.specialization.is_some())
        {
            let (unsafe_, applicable) = read(self, report, definition)?;
            // The sizing profile owns the applicable flag and exact deny-impl contract.
            if unsafe_.is_none()
                && applicable.is_some()
                && self
                    .source_bindings
                    .as_deref()
                    .map(|catalog| catalog.sizing_kind(definition))
                    .transpose()
                    .map_err(Error::unlocated)?
                    .flatten()
                    .is_some()
            {
                return Ok(view);
            }
            if let Some(marker) = unsafe_.or(applicable) {
                return Err(report.diagnostic(Error::at(
                    marker.owner.file,
                    marker.span.start,
                    if unsafe_.is_some() {
                        "source unsafe-specialization-marker contract transport is not implemented"
                    } else {
                        "source specialization-trait contract transport is not implemented"
                    },
                )));
            }
        }
        Ok(view)
    }
    pub(crate) fn reject_specialization_transport(
        &self,
        report: &Report,
        definition: DefId,
    ) -> Result<(), Error> {
        let actual = report
            .resolution
            .definitions
            .get(definition.0 as usize)
            .ok_or_else(|| Error::unlocated("specialization publication header is missing"))?;
        if actual.kind == DefKind::Trait {
            self.publication_header_contracts(report, definition)?;
        }
        Ok(())
    }
}
