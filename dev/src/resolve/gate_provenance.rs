//! Actual sparse sites retain one source incarnation and a sealed token context.
use super::{
    package::{Error, Package},
    unit::{FileId, NodeKey},
};
use crate::syntax::gate_sites::{Declaration, Kind};

fn parsed<'a>(
    package: &'a Package,
    key: NodeKey,
    declaration: &Declaration,
) -> Result<&'a crate::syntax::item::Item, Error> {
    let invalid = || {
        Error::at(
            key.file,
            declaration.span.start,
            "defaultness source owner changed",
        )
    };
    let unit = package.units.get(key.file.0 as usize).ok_or_else(invalid)?;
    unit.configured
        .ast
        .validate_gate_sites_source(unit.source.original_image())
        .map_err(|error| Error::at(key.file, declaration.span.start, error.message))?;
    declaration
        .validate(&unit.configured.ast)
        .map_err(|error| Error::at(key.file, error.offset, error.message))?;
    if key.node != declaration.node
        || declaration.kind != Kind::TraitAlias
            && unit
                .source
                .text()
                .get(declaration.span.start as usize..declaration.span.end as usize)
                != Some(if declaration.kind == Kind::ForeignType { "type" } else { "default" })
    {
        return Err(invalid());
    }
    unit.configured
        .ast
        .items
        .get(declaration.item as usize)
        .ok_or_else(invalid)
}
fn context(
    package: &Package,
    file: FileId,
    declaration: &Declaration,
) -> Result<crate::expand::marks::SyntaxContextId, Error> {
    let entries = package
        .unit(file)
        .configured
        .ast
        .gate_sites
        .as_ref()
        .ok_or_else(|| {
            Error::at(
                file,
                declaration.span.start,
                "defaultness column disappeared",
            )
        })?;
    let context = super::unit::contexts::at_counted(
        &package.units,
        file,
        declaration.span.start,
        |steps, links, lookups| {
            entries.charge(steps.saturating_add(links).saturating_add(lookups), 0)
        },
    )?;
    entries
        .charge(1, 0)
        .map_err(|message| Error::at(file, declaration.span.start, message))?;
    package
        .marks
        .outer(context)
        .map_err(|message| Error::at(file, declaration.span.start, message))?;
    Ok(context)
}
pub(super) fn checked<'a>(
    package: &'a Package,
    key: NodeKey,
    declaration: &Declaration,
) -> Result<&'a crate::syntax::item::Item, Error> {
    let item = parsed(package, key, declaration)?;
    if !declaration.context_matches(context(package, key.file, declaration)?.key()) {
        return Err(Error::at(
            key.file,
            declaration.span.start,
            "defaultness token context changed or was never sealed",
        ));
    }
    Ok(item)
}
pub(super) fn bind_pending(package: &mut Package, file: FileId) -> Result<(), Error> {
    loop {
        let Some(declaration) = package
            .unit(file)
            .configured
            .ast
            .gate_sites
            .as_ref()
            .and_then(|entries| entries.pending_context())
        else {
            return Ok(());
        };
        parsed(
            package,
            NodeKey {
                file,
                node: declaration.node,
            },
            &declaration,
        )?;
        let actual = context(package, file, &declaration)?.key();
        package.units[file.0 as usize]
            .configured
            .ast
            .gate_sites
            .as_mut()
            .ok_or_else(|| {
                Error::at(
                    file,
                    declaration.span.start,
                    "pending defaultness column disappeared",
                )
            })?
            .seal_context(declaration, actual)
            .map_err(|message| Error::at(file, declaration.span.start, message))?;
    }
}
pub(super) fn prepare_plain(package: &mut Package) -> Result<(), Error> {
    for index in 0..package.units.len() {
        let file = FileId(
            u32::try_from(index)
                .map_err(|_| Error::unlocated("defaultness file identity overflow"))?,
        );
        bind_pending(package, file)?;
    }
    Ok(())
}
