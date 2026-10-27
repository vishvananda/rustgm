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
                != Some(match declaration.kind {
                    Kind::ForeignType => "type",
                    Kind::FinalFunction => "final",
                    _ => "default",
                })
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
    span: crate::syntax::token_tree::Span,
) -> Result<crate::expand::marks::SyntaxContextId, Error> {
    let entries = package
        .unit(file)
        .configured
        .ast
        .gate_sites
        .as_ref()
        .ok_or_else(|| Error::at(file, span.start, "defaultness column disappeared"))?;
    let context = super::unit::contexts::at_counted(
        &package.units,
        file,
        span.start,
        |steps, links, lookups| {
            entries.charge(steps.saturating_add(links).saturating_add(lookups), 0)
        },
    )?;
    entries
        .charge(1, 0)
        .map_err(|message| Error::at(file, span.start, message))?;
    package
        .marks
        .outer(context)
        .map_err(|message| Error::at(file, span.start, message))?;
    Ok(context)
}
pub(super) fn checked<'a>(
    package: &'a Package,
    key: NodeKey,
    declaration: &Declaration,
) -> Result<&'a crate::syntax::item::Item, Error> {
    let item = parsed(package, key, declaration)?;
    if !declaration.context_matches(context(package, key.file, declaration.span)?.key()) {
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
            return bind_expressions(package, file);
        };
        parsed(
            package,
            NodeKey {
                file,
                node: declaration.node,
            },
            &declaration,
        )?;
        let actual = context(package, file, declaration.span)?.key();
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
fn parsed_expression(
    package: &Package,
    file: FileId,
    ordinal: u32,
    expression: crate::syntax::gate_sites::Expression,
) -> Result<(), Error> {
    let invalid = || {
        Error::at(
            file,
            expression.keyword.start,
            "expression-gate source site changed",
        )
    };
    let unit = package.units.get(file.0 as usize).ok_or_else(invalid)?;
    unit.configured
        .ast
        .validate_gate_sites_source(unit.source.original_image())
        .map_err(|error| Error::at(file, error.offset, error.message))?;
    expression
        .validate(&unit.configured.ast, ordinal)
        .map_err(|error| Error::at(file, error.offset, error.message))?;
    if let crate::syntax::ast::Kind::OffsetOf {
        expression: original,
        fields,
        ..
    } = unit.configured.ast.node(expression.node).kind
    {
        let entries = unit
            .configured
            .ast
            .gate_sites
            .as_ref()
            .ok_or_else(invalid)?;
        crate::syntax::offset_fields::validate_source(
            &unit.configured.ast,
            original,
            fields,
            unit.source.text(),
            |steps| entries.charge(steps, 0),
        )
        .map_err(|error| Error::at(file, error.offset, error.message))?;
    }
    if unit.configured.ast.expression_gate(ordinal) != Some(&expression)
        || unit
            .source
            .text()
            .get(expression.keyword.start as usize..expression.keyword.end as usize)
            != Some(
                expression
                    .kind(&unit.configured.ast)
                    .map_err(|error| Error::at(file, error.offset, error.message))?
                    .keyword(),
            )
    {
        return Err(invalid());
    }
    Ok(())
}
pub(super) fn checked_expression(
    package: &Package,
    file: FileId,
    ordinal: u32,
    expression: crate::syntax::gate_sites::Expression,
) -> Result<(), Error> {
    parsed_expression(package, file, ordinal, expression)?;
    let actual = (
        context(package, file, expression.keyword)?.key(),
        context(package, file, expression.construct)?.key(),
    );
    if expression.context_key() != Some(actual) {
        return Err(Error::at(
            file,
            expression.keyword.start,
            "expression-gate endpoint contexts changed or were never sealed",
        ));
    }
    Ok(())
}
fn bind_expressions(package: &mut Package, file: FileId) -> Result<(), Error> {
    loop {
        let Some((ordinal, expression)) = package
            .unit(file)
            .configured
            .ast
            .gate_sites
            .as_ref()
            .and_then(|entries| entries.pending_expression())
        else {
            return Ok(());
        };
        parsed_expression(package, file, ordinal, expression)?;
        let actual = (
            context(package, file, expression.keyword)?.key(),
            context(package, file, expression.construct)?.key(),
        );
        package.units[file.0 as usize]
            .configured
            .ast
            .gate_sites
            .as_mut()
            .ok_or_else(|| {
                Error::at(
                    file,
                    expression.keyword.start,
                    "pending expression-gate owner disappeared",
                )
            })?
            .seal_expression(ordinal, expression, actual)
            .map_err(|message| Error::at(file, expression.keyword.start, message))?;
    }
}
