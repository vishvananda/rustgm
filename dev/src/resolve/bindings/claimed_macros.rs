//! Configured source claims select existing expansion contracts, never stub bodies.
use super::policy::Feature;
use crate::{
    expand::builtin::Builtin,
    lex::token::Punctuation,
    resolve::{
        package::{Error, Package},
        unit::NodeKey,
    },
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input},
        item::ItemKind,
        token_tree::{Delimiter, Forest, TreeKind},
    },
};

pub(super) fn parse(
    package: &Package,
    key: NodeKey,
    attribute: Attribute,
    forest: &Forest,
) -> Result<Builtin, Error> {
    let unit = package.unit(key.file);
    let error = |message| Error::at(key.file, attribute.span.start, message);
    if attribute.unsafe_wrapper {
        return Err(error("builtin macro claim cannot use an unsafe wrapper"));
    }
    let source = package
        .source_core()
        .ok_or_else(|| error("builtin macro binding requires an explicit source-core provider"))?;
    if !source.features.has(Feature::RustcAttributes) {
        return Err(error("builtin macro binding requires rustc_attrs"));
    }
    let Kind::Item(index) = unit.configured.ast.node(key.node).kind else {
        return Err(error("builtin macro binding requires a macro declaration"));
    };
    let item = unit.configured.ast.items[index as usize];
    let span = item
        .name
        .ok_or_else(|| error("builtin macro declaration has no name"))?;
    let declared = unit
        .spelling(span)
        .map_err(|e| Error::at(key.file, e.offset, e.message))?;
    let name = argument(unit.source.text(), forest, attribute)
        .map_err(&error)?
        .unwrap_or_else(|| declared.strip_prefix("r#").unwrap_or(declared));
    let role = Builtin::ALL
        .into_iter()
        .find(|builtin| builtin.name() == name)
        .ok_or_else(|| error("source builtin macro expansion is not implemented"))?;
    if !matches!(item.kind, ItemKind::MacroRules { .. })
        && !(role == Builtin::CfgSelect && matches!(item.kind, ItemKind::DeclarativeMacro { .. }))
    {
        return Err(error(
            "builtin macro binding requires its supported declaration kind",
        ));
    }
    Ok(role)
}

fn argument<'a>(
    text: &'a str,
    forest: &Forest,
    attribute: Attribute,
) -> Result<Option<&'a str>, &'static str> {
    if attribute.input == Input::Empty {
        return Ok(None);
    }
    let error = "invalid source builtin macro arguments";
    let Input::Delimited(group) = attribute.input else {
        return Err(error);
    };
    if !forest.tree(group).is_some_and(|tree| {
        matches!(
            tree.kind,
            TreeKind::Group {
                delimiter: Delimiter::Parenthesis,
                ..
            }
        )
    }) {
        return Err(error);
    }
    let range = forest.children(group).ok_or(error)?;
    let name = forest.name(text, range.start).ok_or(error)?;
    let mut cursor = range.start + 1;
    if cursor < range.end {
        if !forest.punctuation(cursor, Punctuation::Comma) {
            return Err(error);
        }
        cursor += 1;
    }
    if cursor < range.end {
        if forest.name(text, cursor) != Some("attributes") {
            return Err(error);
        }
        cursor += 1;
        let helpers = forest.children(cursor).ok_or(error)?;
        if !forest.tree(cursor).is_some_and(|tree| {
            matches!(
                tree.kind,
                TreeKind::Group {
                    delimiter: Delimiter::Parenthesis,
                    ..
                }
            )
        }) {
            return Err(error);
        }
        if helpers.start != helpers.end {
            return Err("source builtin macro helper attributes are not implemented");
        }
        cursor = forest.after(cursor);
        if cursor < range.end && forest.punctuation(cursor, Punctuation::Comma) {
            cursor += 1;
        }
    }
    if cursor != range.end {
        return Err(error);
    }
    Ok(Some(name))
}

#[cfg(test)]
#[path = "../../../tests/invariants/claimed_macro_bindings.rs"]
mod tests;
