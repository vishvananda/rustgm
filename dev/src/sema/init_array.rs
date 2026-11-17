//! Bind a GNU initializer section to the exact configured static attribute.
use crate::{
    literal::{self, Value},
    resolve::{expansion::Report, package::Error, unit::NodeKey},
    sema::model::Declarations,
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input, Style},
        item::ItemKind,
        token_tree::Forest,
    },
};

pub(super) fn section(
    report: &Report,
    facts: &mut Declarations,
    declaration: NodeKey,
    attribute: Attribute,
    forest: &Forest,
) -> Result<Box<str>, Error> {
    facts.reserve_must_use_source(declaration, attribute.span)?;
    let error = |message| Error::at(declaration.file, attribute.span.start, message);
    let unit = report.package.unit(declaration.file);
    let ast = &unit.configured.ast;
    if attribute.style != Style::Outer
        || !matches!(ast.node(declaration.node).kind, Kind::Item(index)
            if matches!(ast.items[index as usize].kind, ItemKind::Static { value: Some(_), .. }))
    {
        return Err(error(
            "source init-array section requires an initialized static",
        ));
    }
    if !attribute.unsafe_wrapper {
        let token = forest
            .token(attribute.path.start)
            .ok_or_else(|| error("link_section lost its actual name token"))?;
        let edition = crate::resolve::unit::editions::token(declaration.file, token, |file| {
            report.package.units.get(file.0 as usize)
        })
        .map_err(|failure| error(failure.message))?;
        if !edition.before_2024() {
            return Err(error("unsafe attribute used without unsafe"));
        }
    }
    let Input::Equals(range) = attribute.input else {
        return Err(error("link_section requires a string literal"));
    };
    if range.end != range.start + 1 {
        return Err(error("link_section requires a string literal"));
    }
    let token = forest
        .token(range.start)
        .ok_or_else(|| error("link_section requires a string literal"))?;
    let Some(Value::String(bytes)) =
        literal::interpret(unit.source.text(), token).map_err(|failure| error(failure.message))?
    else {
        return Err(error("link_section requires a string literal"));
    };
    let name = std::str::from_utf8(bytes.as_ref()).map_err(|_| error("invalid section string"))?;
    if !crate::support::linkage::gnu_init_array_section(name) {
        return Err(error("only GNU init-array global sections are supported"));
    }
    crate::sema::header_attributes::reserve_source_contracts(facts, name.len(), name.len())?;
    Ok(name.into())
}
