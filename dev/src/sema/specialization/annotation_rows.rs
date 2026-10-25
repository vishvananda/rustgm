//! The existing configured-owner and definition passes produce all local rows once.
use super::annotation_store::{self as store, Marker, Owner};
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind},
        package::Error,
        unit::NodeKey,
    },
    sema::model::Declarations,
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input},
        item::ItemKind,
    },
};
pub(in crate::sema) fn prepare(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    if report.package.units.iter().any(|unit| {
        unit.configured
            .ast
            .defaultness
            .as_ref()
            .is_some_and(|rows| rows.len() != 0)
    }) {
        store::owner(facts)?;
    }
    Ok(())
}
pub(in crate::sema) fn observe_owner(
    report: &Report,
    facts: &mut Declarations,
    owner: NodeKey,
    declaration: NodeKey,
) -> Result<(), Error> {
    if !facts
        .must_use_contracts
        .as_deref()
        .is_some_and(|table| table.specialization.is_some())
        || !matches!(report.package.unit(declaration.file).configured.ast.node(declaration.node).kind,
            Kind::Item(index) if matches!(report.package.unit(declaration.file).configured.ast.items[index as usize].kind, ItemKind::Trait { .. }))
    {
        return Ok(());
    }
    let unit = report.package.unit(owner.file);
    let actual = unit
        .owner_indices
        .get(owner.node.get() as usize)
        .copied()
        .flatten()
        .and_then(|index| unit.configured.owners.get(index.get() as usize - 1))
        .ok_or_else(|| Error::at(owner.file, 0, "specialization marker lost configured owner"))?;
    if owner.file != declaration.file || !actual.enabled || actual.node != owner.node {
        return Err(Error::at(
            owner.file,
            0,
            "specialization marker owner is inactive or foreign",
        ));
    }
    let table = store::owner(facts)?;
    let row = table.row(report, declaration)?;
    let append = row.owners.last().is_none_or(|last| last.key != owner);
    let growth = crate::support::retained::Growth::vector::<Owner>(
        row.owners.len(),
        row.owners.capacity(),
        usize::from(append),
    );
    table
        .charge(1 + growth.work, growth.bytes)
        .map_err(|message| Error::at(owner.file, 0, message))?;
    if append {
        let row = table.sources.get_mut(&declaration).ok_or_else(|| {
            Error::at(
                owner.file,
                0,
                "specialization trait row disappeared after payment",
            )
        })?;
        row.owners.push(Owner {
            key: owner,
            syntax: unit.configured.ast.node(owner.node).kind,
            effects: actual.attributes.clone(),
        });
    }
    Ok(())
}
pub(in crate::sema) fn consume(
    report: &Report,
    facts: &mut Declarations,
    owner: NodeKey,
    declaration: NodeKey,
    attribute: Attribute,
    effect: usize,
) -> Result<(), Error> {
    let error = |message| Error::at(owner.file, attribute.span.start, message);
    if attribute.input != Input::Empty || attribute.unsafe_wrapper {
        return Err(error(
            "rustc_unsafe_specialization_marker requires word form",
        ));
    }
    store::trait_syntax(report, declaration)?;
    let unit = report.package.unit(owner.file);
    let (retained, forest) = unit
        .retained_attributes(effect..effect + 1)
        .next()
        .ok_or_else(|| error("specialization marker lost retained capture"))?;
    let name = forest
        .tree(attribute.path.start)
        .ok_or_else(|| error("specialization marker lost name token"))?
        .span;
    if retained.span != attribute.span || retained.path != attribute.path {
        return Err(error("specialization marker capture changed"));
    }
    if !crate::sema::parentheses::feature(
        report,
        &mut report.macro_allowances(),
        (owner.file, name),
        false,
    )? {
        return Err(error(
            "rustc_unsafe_specialization_marker requires rustc_attrs",
        ));
    }
    store::owner(facts)?;
    observe_owner(report, facts, owner, declaration)?;
    let table = store::owner(facts)?;
    table.charge(1, 0).map_err(error)?;
    let row = table.row(report, declaration)?;
    if row.marker.is_some() {
        return Err(error("duplicate rustc_unsafe_specialization_marker"));
    }
    row.marker = Some(Marker {
        owner,
        effect,
        span: attribute.span,
        name,
    });
    Ok(())
}
pub(in crate::sema) fn bind_trait(
    report: &Report,
    facts: &mut Declarations,
    definition: DefId,
    declaration: NodeKey,
) -> Result<(), Error> {
    let Some(table) = facts
        .must_use_contracts
        .as_deref_mut()
        .and_then(|contracts| contracts.specialization.as_deref_mut())
    else {
        return Ok(());
    };
    let actual = report
        .resolution
        .definitions
        .get(definition.0 as usize)
        .ok_or_else(|| {
            Error::at(
                declaration.file,
                0,
                "specialization trait binding is missing",
            )
        })?;
    if actual.kind != DefKind::Trait {
        return Ok(());
    }
    if report
        .package
        .unit(declaration.file)
        .include_context
        .is_some()
    {
        return Ok(());
    }
    if actual.node != Some(declaration) || table.complete {
        return Err(Error::at(
            declaration.file,
            0,
            "specialization trait binding changed",
        ));
    }
    let growth = crate::support::retained::Growth::map::<DefId, NodeKey>(
        table.definitions.len(),
        table.definitions.capacity(),
        usize::from(!table.definitions.contains_key(&definition)),
    );
    table
        .charge(1 + growth.work, growth.bytes)
        .map_err(|message| Error::at(declaration.file, 0, message))?;
    table.row(report, declaration)?;
    if table.definitions.insert(definition, declaration).is_some() {
        return Err(Error::at(
            declaration.file,
            0,
            "specialization trait was bound twice",
        ));
    }
    Ok(())
}
pub(in crate::sema) fn finish(facts: &mut Declarations) -> Result<(), Error> {
    if let Some(table) = facts
        .must_use_contracts
        .as_deref_mut()
        .and_then(|contracts| contracts.specialization.as_deref_mut())
    {
        table
            .charge(table.sources.len(), 0)
            .map_err(Error::unlocated)?;
        if table.sources.len() != table.definitions.len() {
            return Err(Error::unlocated(
                "specialization marker has an unbound source trait",
            ));
        }
        table.complete = true;
    }
    Ok(())
}
