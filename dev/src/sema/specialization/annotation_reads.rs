//! Missing or imported trait state is Unknown, never a local marker absence.
use super::annotation_store::{self as store, Marker};
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
    Ok(read(facts, report, definition)?.is_some())
}
fn read(facts: &Declarations, report: &Report, definition: DefId) -> Result<Option<Marker>, Error> {
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
        validate_marker(report, marker)?;
    }
    Ok(row.marker)
}
fn validate_marker(report: &Report, marker: Marker) -> Result<(), Error> {
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
        || attribute.name(forest, unit.source.text()) != Some("rustc_unsafe_specialization_marker")
        || forest
            .tree(attribute.path.start)
            .is_none_or(|tree| tree.span != marker.name)
    {
        return Err(error(
            "specialization marker actual source contract changed",
        ));
    }
    if !crate::sema::parentheses::feature(
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
    /// Publication requires completed source producers before rejecting untransported policy.
    pub(crate) fn publication_header_contracts<'a>(
        &'a self,
        report: &Report,
        definition: DefId,
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
        // Completion is produced after the shared marker consumer finishes.
        // This is a presence fence; the wire still conveys no marker knowledge.
        if self
            .must_use_contracts
            .as_deref()
            .is_some_and(|owner| owner.specialization.is_some())
            && let Some(marker) = read(self, report, definition)?
        {
            return Err(report.diagnostic(Error::at(
                marker.owner.file,
                marker.span.start,
                "source unsafe-specialization-marker contract transport is not implemented",
            )));
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
