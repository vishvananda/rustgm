//! Retained linkage attributes decode literal tokens once into the owning exact-name pool.
#[cfg(test)]
#[path = "../../../../tests/invariants/linkage_editions.rs"]
mod tests;
use crate::support::hash::HashMap;
use crate::{
    literal::{self, Value},
    resolve::{
        expansion::Report,
        package::Error,
        unit::{FileId, NodeKey},
    },
    sema::foreign::Registry,
    support::linkage::SymbolNameId,
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input},
        item::ItemKind,
        token_tree::Forest,
    },
};
#[derive(Clone, Copy, Default)]
pub(in crate::sema::declarations) struct Attributes {
    pub link: Option<SymbolNameId>,
    pub export: Option<SymbolNameId>,
    pub no_mangle: bool,
}
pub(in crate::sema::declarations) fn collect(
    report: &Report,
    registry: &mut Option<Box<Registry>>,
) -> Result<HashMap<NodeKey, Attributes>, Error> {
    let mut sources = HashMap::default();
    for (file, unit) in report.package.units.iter().enumerate() {
        if unit.include_context.is_some() {
            continue;
        }
        let file = FileId(
            u32::try_from(file)
                .map_err(|_| Error::at(FileId(0), 0, "too many linkage source files"))?,
        );
        for owner in &unit.configured.owners {
            if !owner.enabled || owner.attributes.is_empty() {
                continue;
            }
            let mut node = owner.node;
            while let Kind::Attributed { value, .. } = unit.configured.ast.node(node).kind {
                node = value;
            }
            for (attribute, forest) in unit.retained_attributes(owner.attributes.clone()) {
                let Some(name @ ("link_name" | "export_name" | "no_mangle")) =
                    attribute.name(forest, unit.source.text())
                else {
                    continue;
                };
                let error = |message| Error::at(file, attribute.span.start, message);
                let Kind::Item(index) = unit.configured.ast.node(node).kind else {
                    return Err(error(
                        "linkage attribute requires a function or static item",
                    ));
                };
                if !matches!(
                    unit.configured.ast.items[index as usize].kind,
                    ItemKind::Function { .. } | ItemKind::Static { .. }
                ) {
                    return Err(error(
                        "linkage attribute requires a function or static item",
                    ));
                }
                safety(report, file, forest, attribute, name)?;
                let selected = sources
                    .entry(NodeKey { file, node })
                    .or_insert_with(Attributes::default);
                if name == "no_mangle" {
                    if attribute.input != Input::Empty {
                        return Err(error("no_mangle requires no arguments"));
                    }
                    selected.no_mangle = true;
                } else {
                    let id =
                        exact(unit.source.text(), forest, attribute, registry).map_err(error)?;
                    let slot = if name == "link_name" {
                        &mut selected.link
                    } else {
                        &mut selected.export
                    };
                    slot.get_or_insert(id);
                }
            }
        }
    }
    Ok(sources)
}
fn safety(
    report: &Report,
    file: FileId,
    forest: &Forest,
    attribute: Attribute,
    name: &str,
) -> Result<(), Error> {
    if attribute.unsafe_wrapper == (name != "link_name") {
        return Ok(());
    }
    let error = |message| Error::at(file, attribute.span.start, message);
    if !attribute.unsafe_wrapper && name != "link_name" {
        // The name owns the edition; a macro may supply the surrounding #[...]
        // from another edition. Retain the parsed token's exact source ancestry.
        let token = forest
            .token(attribute.path.start)
            .ok_or_else(|| error("missing linkage attribute name token"))?;
        let edition = crate::resolve::unit::editions::token(file, token, |file| {
            report.package.units.get(file.0 as usize)
        })
        .map_err(|failure| error(failure.message))?;
        if edition == crate::support::edition::Edition::Rust2021 {
            return Ok(());
        }
    }
    Err(error("invalid unsafe wrapper for linkage attribute"))
}
fn exact(
    source: &str,
    forest: &Forest,
    attribute: Attribute,
    registry: &mut Option<Box<Registry>>,
) -> Result<SymbolNameId, &'static str> {
    let Input::Equals(range) = attribute.input else {
        return Err("linkage attribute requires a string literal");
    };
    if range.end != range.start + 1 {
        return Err("linkage attribute requires one string literal");
    }
    let token = forest
        .token(range.start)
        .ok_or("linkage attribute requires a literal token")?;
    let Some(Value::String(bytes)) = literal::interpret(source, token).map_err(|e| e.message)?
    else {
        return Err("linkage attribute requires a string literal");
    };
    let spelling =
        std::str::from_utf8(bytes.as_ref()).map_err(|_| "linkage attribute is not valid UTF-8")?;
    registry
        .get_or_insert_default()
        .names_mut()
        .intern(spelling)
}
